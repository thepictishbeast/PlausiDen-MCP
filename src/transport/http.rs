//! HTTP+JSON transport — POST /messages over plain HTTP, localhost only.
//!
//! This is the optional alternative to the stdio transport. It is gated
//! behind the `http` Cargo feature and pulls in axum + tower as optional
//! dependencies.
//!
//! ## Security model
//!
//! The HTTP transport ALWAYS binds to `127.0.0.1` only — it never accepts
//! connections from any non-loopback address. This is enforced in code,
//! not in config, so a future config typo can't accidentally open the
//! server to the world. The MCP capability gate (audit-logged, six groups,
//! Inject/Swarm requiring typed acknowledgment) still applies on top.
//!
//! Per AVP-2 / PlausiDen master standards: this transport is NOT exposed
//! beyond loopback. Anyone wanting remote access must front it with a
//! mutual-TLS tunnel (e.g. WireGuard + mTLS) — exposing this directly to
//! the internet is a misconfiguration.
//!
//! ## Endpoints
//!
//! | Method | Path        | Body            | Response                      |
//! |--------|-------------|-----------------|-------------------------------|
//! | GET    | `/health`   | —               | `200 {"status":"ok",...}`     |
//! | POST   | `/messages` | JSON-RPC 2.0    | JSON-RPC 2.0 response         |
//!
//! Server-Sent Events (SSE) for server→client notifications is intentionally
//! NOT implemented yet — the current MCP implementation is request/response
//! only and has no notification surface that warrants SSE complexity. When
//! resources/subscribe lands, the SSE endpoint will be added here.
//!
//! ## AVP-2 stop point
//!
//! Tier 1 (Existence Proof, passes 1–6) is the minimum bar for landing this
//! file. Verified inline:
//! - Pass 1 (compiles + runs): `cargo build --features http`
//! - Pass 2 (null path):       empty POST body → JSON-RPC parse error 32700
//! - Pass 3 (boundary):        body capped at `MAX_BODY_BYTES` (1 MiB)
//! - Pass 4 (error path):      malformed JSON returns parse error not 500
//! - Pass 5 (static analysis): `cargo clippy -- -D warnings` clean
//! - Pass 6 (concurrency):     each request handled in its own tokio task
//!   (axum default); McpServer state is shared via Arc and is internally
//!   synchronized.
//!
//! Tier 2+ (failure resilience, adversarial security, mutation testing)
//! is deferred until this transport is actually used in production. The
//! stdio transport remains the primary production path.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::audit::AuditEvent;
use crate::server::{JsonRpcRequest, JsonRpcResponse, McpServer};

/// Maximum accepted POST body size, in bytes. The MCP spec doesn't pin a
/// hard limit but JSON-RPC requests that exceed 1 MiB are pathological for
/// a local-IPC-style transport — the request is almost certainly an attack
/// or a bug in the client. Reject early to keep memory bounded.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Loopback address the HTTP transport binds to. Hard-coded to `127.0.0.1`
/// (NEVER `0.0.0.0`) so the server cannot be misconfigured into accepting
/// non-loopback traffic. See module docs for the rationale.
const BIND_ADDR: &str = "127.0.0.1:8765";

/// Run the HTTP transport.
///
/// Binds to [`BIND_ADDR`] and serves the routes defined in [`router`]. Blocks
/// the calling task until the listener is shut down (e.g. by Ctrl+C from the
/// surrounding tokio runtime).
pub async fn run(server: Arc<McpServer>) -> Result<(), Box<dyn std::error::Error>> {
    server.audit.record(AuditEvent::ServerStart);

    let addr: SocketAddr = BIND_ADDR.parse().expect("BIND_ADDR is a valid SocketAddr");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "plausiden-mcp HTTP transport listening (loopback only)");

    let app = router(server.clone());
    axum::serve(listener, app).await?;

    server.audit.record(AuditEvent::ServerStop);
    Ok(())
}

/// Construct the axum router with the McpServer as shared state.
///
/// Split out from [`run`] so it is unit-testable without binding a port.
pub fn router(server: Arc<McpServer>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/messages", post(messages))
        .with_state(server)
}

/// `GET /health` — liveness probe.
///
/// Returns 200 OK with the server name and version. No capability gating
/// is applied because no McpServer state is consulted; this is a pure
/// liveness check usable by monitoring without needing any privileges.
async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

/// `POST /messages` — JSON-RPC 2.0 request handler.
///
/// Reads up to [`MAX_BODY_BYTES`] of body, parses it as a [`JsonRpcRequest`],
/// dispatches it through [`McpServer::handle_request`], and returns the
/// response as JSON. Parse failures return a JSON-RPC parse error (-32700)
/// over HTTP 200 — that's the JSON-RPC convention, NOT a 4xx status, so the
/// client always gets a structured error envelope it can handle uniformly.
async fn messages(
    State(server): State<Arc<McpServer>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if body.len() > MAX_BODY_BYTES {
        // We can't return a JSON-RPC error here without knowing the request
        // id, and the body is also too large to safely deserialize. Return
        // a 413 with a minimal JSON envelope so the client can detect it.
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "error": "request body exceeds maximum size",
                "max_bytes": MAX_BODY_BYTES,
            })),
        )
            .into_response();
    }

    let response: JsonRpcResponse = match serde_json::from_slice::<JsonRpcRequest>(&body) {
        Ok(request) => server.handle_request(request).await,
        Err(e) => JsonRpcResponse::error(Value::Null, -32700, format!("Parse error: {e}")),
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot`

    /// Build a fresh McpServer + router pair for a single test.
    fn fixture() -> Router {
        let server = Arc::new(McpServer::new(ServerConfig::default()));
        router(server)
    }

    /// Drain a response body into bytes for assertion.
    async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
        to_bytes(resp.into_body(), MAX_BODY_BYTES + 1)
            .await
            .expect("body collected")
            .to_vec()
    }

    /// Pass 1 (Existence Proof): the router builds with a fresh McpServer.
    #[test]
    fn router_constructs_with_default_server() {
        let _router = fixture();
    }

    /// Pass 3 (Boundary): the maximum body size constant is sane.
    #[test]
    fn max_body_bytes_is_one_mib() {
        assert_eq!(MAX_BODY_BYTES, 1024 * 1024);
    }

    /// Pass 1 (Existence Proof): bind address is parseable as a SocketAddr.
    #[test]
    fn bind_addr_is_loopback() {
        let addr: SocketAddr = BIND_ADDR.parse().expect("valid socket addr");
        assert!(addr.ip().is_loopback(), "BIND_ADDR must be loopback");
    }

    /// Pass 1 (Existence Proof): GET /health returns 200 with the expected
    /// service identity. This is the liveness probe used by uptime monitors,
    /// so it must work without any capability gating or McpServer state read.
    #[tokio::test]
    async fn health_returns_200_with_service_identity() {
        let app = fixture();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .expect("request built");

        let resp = app.oneshot(req).await.expect("oneshot ok");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_bytes(resp).await;
        let json: Value = serde_json::from_slice(&body).expect("body is JSON");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], env!("CARGO_PKG_NAME"));
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    }

    /// Pass 2 (Null Path): an empty POST /messages body must return a
    /// JSON-RPC parse error (-32700) over HTTP 200, NOT a 4xx or 500.
    /// Per JSON-RPC 2.0 convention, parse errors are wrapped in a normal
    /// response envelope so the client can handle them uniformly.
    #[tokio::test]
    async fn messages_empty_body_returns_jsonrpc_parse_error() {
        let app = fixture();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .header("content-type", "application/json")
            .body(Body::empty())
            .expect("request built");

        let resp = app.oneshot(req).await.expect("oneshot ok");
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "JSON-RPC parse errors must use HTTP 200, not 4xx/5xx"
        );

        let body = body_bytes(resp).await;
        let json: Value = serde_json::from_slice(&body).expect("body is JSON");
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(
            json["error"]["code"], -32700,
            "JSON-RPC 2.0 parse error code is -32700"
        );
    }

    /// Pass 4 (Error Path): malformed JSON in POST /messages must surface as
    /// a JSON-RPC parse error (-32700), NOT an HTTP 500 panic. The transport
    /// is responsible for catching every deserialization failure and turning
    /// it into a structured envelope.
    #[tokio::test]
    async fn messages_malformed_json_returns_jsonrpc_parse_error() {
        let app = fixture();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .header("content-type", "application/json")
            .body(Body::from("{this is not valid json"))
            .expect("request built");

        let resp = app.oneshot(req).await.expect("oneshot ok");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_bytes(resp).await;
        let json: Value = serde_json::from_slice(&body).expect("body is JSON");
        assert_eq!(json["error"]["code"], -32700);
        // The error message should reference parse failure, not panic.
        assert!(
            json["error"]["message"]
                .as_str()
                .is_some_and(|s| s.contains("Parse error")),
            "expected 'Parse error' in error.message, got: {}",
            json["error"]["message"]
        );
    }

    /// Pass 3 (Boundary): a body that exceeds MAX_BODY_BYTES must be
    /// rejected with HTTP 413 PAYLOAD_TOO_LARGE before deserialization is
    /// attempted. This bounds memory consumption per request — without
    /// this, an attacker could OOM the server with a single large POST.
    #[tokio::test]
    async fn messages_oversized_body_returns_413() {
        let app = fixture();
        // 1 MiB + 1 byte — just past the limit.
        let oversized = vec![b'x'; MAX_BODY_BYTES + 1];
        let req = Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .header("content-type", "application/json")
            .body(Body::from(oversized))
            .expect("request built");

        let resp = app.oneshot(req).await.expect("oneshot ok");
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let body = body_bytes(resp).await;
        let json: Value = serde_json::from_slice(&body).expect("body is JSON");
        assert_eq!(json["max_bytes"], MAX_BODY_BYTES);
        assert!(json["error"].as_str().is_some());
    }

    /// Pass 1 (Existence Proof): a valid JSON-RPC `initialize` request
    /// round-trips through the router and returns a well-formed
    /// JSON-RPC 2.0 response. This is the smoke test that the
    /// HTTP↔McpServer wiring is end-to-end correct.
    #[tokio::test]
    async fn messages_initialize_roundtrips_to_mcp_server() {
        let app = fixture();
        let body = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "http-test", "version": "0.0.0" }
            }
        }))
        .expect("body serialized");

        let req = Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("request built");

        let resp = app.oneshot(req).await.expect("oneshot ok");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_bytes(resp).await;
        let json: Value = serde_json::from_slice(&body).expect("body is JSON");
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        // initialize must return a result, not an error.
        assert!(
            json.get("result").is_some(),
            "initialize should return a result envelope, got: {json}"
        );
        assert!(json.get("error").is_none());
    }
}
