//! MCP JSON-RPC protocol implementation.
//!
//! Handles stdin/stdout communication following the Model Context Protocol spec.
//! Parses incoming JSON-RPC requests, routes them to tool handlers, and sends responses.

use crate::tools;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

/// MCP Server handling the JSON-RPC protocol over stdin/stdout.
pub struct McpServer {
    tools: tools::ToolRegistry,
}

/// JSON-RPC request envelope.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC response envelope.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

/// MCP tool definition for the tools/list response.
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP tool call result content.
#[derive(Debug, Serialize)]
pub struct ToolResultContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl McpServer {
    /// Create a new MCP server with all tools registered.
    pub fn new() -> Self {
        Self {
            tools: tools::ToolRegistry::new(),
        }
    }

    /// Run the server, reading from stdin and writing to stdout.
    pub async fn run(&self) -> anyhow::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to parse request: {e}");
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;
            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    /// Route a request to the appropriate handler.
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params).await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Handle the initialize request.
    fn handle_initialize(&self, _params: &serde_json::Value) -> Result<serde_json::Value, JsonRpcError> {
        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "plausiden-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    /// Handle tools/list — return all available tools.
    fn handle_tools_list(&self) -> Result<serde_json::Value, JsonRpcError> {
        let tools: Vec<&ToolDefinition> = self.tools.list();
        Ok(serde_json::json!({ "tools": tools }))
    }

    /// Handle tools/call — execute a tool.
    async fn handle_tools_call(&self, params: &serde_json::Value) -> Result<serde_json::Value, JsonRpcError> {
        let name = params["name"].as_str().ok_or(JsonRpcError {
            code: -32602,
            message: "Missing tool name".to_string(),
        })?;

        let arguments = &params["arguments"];
        let result = self.tools.call(name, arguments).await?;

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": result
            }]
        }))
    }
}
