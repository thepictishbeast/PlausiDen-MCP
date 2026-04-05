//! Integration tests for the PlausiDen MCP server.

use plausiden_mcp::audit::{AuditEvent, AuditLog};
use plausiden_mcp::auth::{Capability, CapabilityStore};
use plausiden_mcp::config::ServerConfig;
use plausiden_mcp::server::{JsonRpcRequest, McpServer};

fn make_server() -> McpServer {
    McpServer::new(ServerConfig::default())
}

fn make_request(method: &str, params: Option<serde_json::Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: serde_json::json!(1),
        method: method.to_string(),
        params,
    }
}

#[tokio::test]
async fn test_initialize() {
    let server = make_server();
    let req = make_request("initialize", None);
    let resp = server.handle_request(req).await;

    let result = resp.result.unwrap();
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "plausiden-mcp");
}

#[tokio::test]
async fn test_tools_list() {
    let server = make_server();
    let req = make_request("tools/list", None);
    let resp = server.handle_request(req).await;

    let result = resp.result.unwrap();
    let tools = result["tools"].as_array().unwrap();
    assert!(tools.len() >= 20, "Should have 20+ tools, got {}", tools.len());

    // Verify system tools are present.
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"plausiden_enable_capability"));
    assert!(names.contains(&"plausiden_generate_browser_history"));
    assert!(names.contains(&"plausiden_inject_browser"));
    assert!(names.contains(&"plausiden_swarm_join"));
}

#[tokio::test]
async fn test_generate_requires_capability() {
    let server = make_server();
    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_generate_browser_history",
            "arguments": { "count": 5 }
        })),
    );
    let resp = server.handle_request(req).await;

    // Should fail — generate capability not enabled.
    assert!(resp.error.is_some());
    assert!(resp.error.unwrap().message.contains("generate"));
}

#[tokio::test]
async fn test_enable_then_generate() {
    let server = make_server();

    // Enable generate capability.
    let enable_req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_enable_capability",
            "arguments": { "capability": "generate" }
        })),
    );
    let resp = server.handle_request(enable_req).await;
    assert!(resp.error.is_none());

    // Now generate should work.
    let gen_req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_generate_browser_history",
            "arguments": { "count": 5, "profile": "journalist" }
        })),
    );
    let resp = server.handle_request(gen_req).await;
    assert!(resp.error.is_none());
    let result = resp.result.unwrap();
    assert!(result["content"].is_array());
}

#[tokio::test]
async fn test_inject_requires_acknowledgment() {
    let server = make_server();

    // Try to enable inject without acknowledgment.
    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_enable_capability",
            "arguments": { "capability": "inject" }
        })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_some());
}

#[tokio::test]
async fn test_inject_with_correct_acknowledgment() {
    let server = make_server();

    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_enable_capability",
            "arguments": {
                "capability": "inject",
                "acknowledgment": "I understand this will modify real data on disk"
            }
        })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
    assert!(server.capabilities.is_enabled(Capability::Inject));
}

#[tokio::test]
async fn test_swarm_requires_acknowledgment() {
    let server = make_server();

    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_enable_capability",
            "arguments": { "capability": "swarm" }
        })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_some());
}

#[tokio::test]
async fn test_resources_list() {
    let server = make_server();
    let req = make_request("resources/list", None);
    let resp = server.handle_request(req).await;

    let result = resp.result.unwrap();
    let resources = result["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 4);
}

#[tokio::test]
async fn test_resource_read_capabilities() {
    let server = make_server();
    let req = make_request(
        "resources/read",
        Some(serde_json::json!({ "uri": "plausiden://capabilities" })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_resource_read_presets() {
    let server = make_server();
    let req = make_request(
        "resources/read",
        Some(serde_json::json!({ "uri": "plausiden://profiles/presets" })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_prompts_list() {
    let server = make_server();
    let req = make_request("prompts/list", None);
    let resp = server.handle_request(req).await;

    let result = resp.result.unwrap();
    let prompts = result["prompts"].as_array().unwrap();
    assert_eq!(prompts.len(), 3);
}

#[tokio::test]
async fn test_prompts_get_quick_start() {
    let server = make_server();
    let req = make_request(
        "prompts/get",
        Some(serde_json::json!({ "name": "quick_start" })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());

    let result = resp.result.unwrap();
    let messages = result["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_audit_chain_integrity() {
    let log = AuditLog::new();
    log.record(AuditEvent::ServerStart);
    log.record(AuditEvent::CapabilityEnabled {
        capability: Capability::Generate,
    });
    log.record(AuditEvent::ToolCalled {
        tool_name: "test".to_string(),
        param_hash: "abc".to_string(),
    });
    log.record(AuditEvent::ServerStop);

    assert!(log.verify_chain().is_ok());
    assert_eq!(log.len(), 4);
}

#[tokio::test]
async fn test_no_network_without_swarm() {
    let store = CapabilityStore::new();
    // Swarm disabled by default.
    assert!(!store.is_enabled(Capability::Swarm));
    // Cannot enable without acknowledgment.
    assert!(store.enable(Capability::Swarm, None).is_err());
}

#[tokio::test]
async fn test_unknown_method() {
    let server = make_server();
    let req = make_request("nonexistent/method", None);
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_list_capabilities_tool() {
    let server = make_server();
    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_list_capabilities",
            "arguments": {}
        })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_disable_capability() {
    let server = make_server();

    // Enable then disable.
    server.capabilities.enable(Capability::Generate, None).unwrap();
    assert!(server.capabilities.is_enabled(Capability::Generate));

    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_disable_capability",
            "arguments": { "capability": "generate" }
        })),
    );
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
    assert!(!server.capabilities.is_enabled(Capability::Generate));
}

#[tokio::test]
async fn test_denied_call_audited() {
    let server = make_server();

    // Call a tool without capability — should be denied and audited.
    let req = make_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "plausiden_inject_browser",
            "arguments": {}
        })),
    );
    let _resp = server.handle_request(req).await;

    let entries = server.audit.entries();
    assert!(entries.iter().any(|e| matches!(&e.event, AuditEvent::ToolDenied { .. })));
}
