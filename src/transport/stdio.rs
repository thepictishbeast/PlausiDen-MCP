//! Stdio transport — JSON-RPC 2.0 over stdin/stdout.
//!
//! This is the primary transport used by Claude Code and Cursor.
//! Each line on stdin is a JSON-RPC request; each response is written
//! as a single line to stdout.

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::audit::AuditEvent;
use crate::server::{JsonRpcRequest, McpServer};

/// Run the stdio transport loop.
pub async fn run(server: Arc<McpServer>) -> Result<(), Box<dyn std::error::Error>> {
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
            Err(e) => crate::server::JsonRpcResponse::error(
                serde_json::Value::Null,
                -32700,
                format!("Parse error: {e}"),
            ),
        };

        let response_bytes = serde_json::to_string(&response)?;
        stdout.write_all(response_bytes.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    server.audit.record(AuditEvent::ServerStop);
    Ok(())
}
