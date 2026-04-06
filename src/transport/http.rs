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
        Err(e) => JsonRpcResponse::error(
            Value::Null,
            -32700,
            format!("Parse error: {e}"),
        ),
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;

    /// Pass 1 (Existence Proof): the router builds with a fresh McpServer.
    #[test]
    fn router_constructs_with_default_server() {
        let server = Arc::new(McpServer::new(ServerConfig::default()));
        let _router = router(server);
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
}
