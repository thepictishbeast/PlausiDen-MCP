//! MCP server — JSON-RPC 2.0 dispatch, tool registry, initialization.
//!
//! Implements the Model Context Protocol server that handles initialize,
//! tools/list, tools/call, resources/list, resources/read, and prompts
//! endpoints per the MCP specification.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::audit::{AuditEvent, AuditLog};
use crate::auth::CapabilityStore;
use crate::config::ServerConfig;
use crate::prompts;
use crate::resources;
use crate::tools;

/// The MCP server. Holds shared state for capability checks, audit logging,
/// and tool dispatch.
pub struct McpServer {
    pub config: ServerConfig,
    pub capabilities: CapabilityStore,
    pub audit: AuditLog,
}

impl McpServer {
    /// Create a new server with the given configuration.
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            capabilities: CapabilityStore::new(),
            audit: AuditLog::new(),
        }
    }

    /// Run the server on stdio (stdin/stdout JSON-RPC).
    pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        let server = Arc::new(self);
        server.audit.record(AuditEvent::ServerStart);

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => server.handle_request(request).await,
                Err(e) => JsonRpcResponse::error(Value::Null, -32700, format!("Parse error: {e}")),
            };

            let response_bytes = serde_json::to_string(&response)?;
            stdout.write_all(response_bytes.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        server.audit.record(AuditEvent::ServerStop);
        Ok(())
    }

    /// Handle a single JSON-RPC request and return a response.
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "initialized" => JsonRpcResponse::success(id, Value::Null),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request.params).await,
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, request.params),
            "prompts/list" => self.handle_prompts_list(id),
            "prompts/get" => self.handle_prompts_get(id, request.params),
            _ => {
                JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method))
            }
        }
    }

    /// MCP initialize — advertise server capabilities.
    fn handle_initialize(&self, id: Value) -> JsonRpcResponse {
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": false },
                "resources": { "subscribe": false, "listChanged": false },
                "prompts": { "listChanged": false },
            },
            "serverInfo": {
                "name": self.config.server_name,
                "version": self.config.server_version,
            }
        });
        JsonRpcResponse::success(id, result)
    }

    /// List all registered tools with their schemas.
    fn handle_tools_list(&self, id: Value) -> JsonRpcResponse {
        let tool_list = tools::list_all_tools();
        let result = serde_json::json!({ "tools": tool_list });
        JsonRpcResponse::success(id, result)
    }

    /// Dispatch a tool call — check capability, audit, execute.
    async fn handle_tools_call(&self, id: Value, params: Option<Value>) -> JsonRpcResponse {
        let params = params.unwrap_or(Value::Null);

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tool_args = params
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        // Check capability requirement for this tool.
        let required_cap = tools::required_capability(&tool_name);
        if let Some(cap) = required_cap {
            if let Err(_e) = self.capabilities.require(cap) {
                self.audit.record(AuditEvent::ToolDenied {
                    tool_name: tool_name.clone(),
                    required_capability: cap,
                });
                return JsonRpcResponse::error(
                    id,
                    -32001,
                    format!(
                        "Capability '{}' is not enabled. Enable it first with plausiden_enable_capability.",
                        cap
                    ),
                );
            }
        }

        // Audit the call.
        let param_hash = AuditLog::hash_params(&tool_args);
        self.audit.record(AuditEvent::ToolCalled {
            tool_name: tool_name.clone(),
            param_hash,
        });

        // Dispatch to the tool handler.
        let result = tools::dispatch(&tool_name, &tool_args, self).await;

        match result {
            Ok(content) => {
                let response = serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                    }]
                });
                JsonRpcResponse::success(id, response)
            }
            Err(e) => {
                self.audit.record(AuditEvent::Error {
                    message: e.to_string(),
                });
                JsonRpcResponse::error(id, -32000, e.to_string())
            }
        }
    }

    /// List available resources.
    fn handle_resources_list(&self, id: Value) -> JsonRpcResponse {
        let resource_list = resources::list_all_resources();
        let result = serde_json::json!({ "resources": resource_list });
        JsonRpcResponse::success(id, result)
    }

    /// Read a specific resource.
    fn handle_resources_read(&self, id: Value, params: Option<Value>) -> JsonRpcResponse {
        let params = params.unwrap_or(Value::Null);
        let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");

        match resources::read_resource(uri, self) {
            Ok(content) => {
                let result = serde_json::json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                    }]
                });
                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(id, -32002, e.to_string()),
        }
    }

    /// List available prompts.
    fn handle_prompts_list(&self, id: Value) -> JsonRpcResponse {
        let prompt_list = prompts::list_all_prompts();
        let result = serde_json::json!({ "prompts": prompt_list });
        JsonRpcResponse::success(id, result)
    }

    /// Get a specific prompt.
    fn handle_prompts_get(&self, id: Value, params: Option<Value>) -> JsonRpcResponse {
        let params = params.unwrap_or(Value::Null);
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let arguments: HashMap<String, String> = params
            .get("arguments")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        match prompts::get_prompt(name, &arguments) {
            Ok(messages) => {
                let result = serde_json::json!({ "messages": messages });
                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(id, -32002, e.to_string()),
        }
    }
}

/// JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    /// Construct a success response.
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Construct an error response.
    pub fn error(id: Value, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}
