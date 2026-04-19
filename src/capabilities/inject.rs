//! Inject capability — DANGEROUS: modifies real data on disk.
//!
//! All inject tools require the Inject capability, which demands typed
//! acknowledgment before enabling. Every injection is audit-logged.

use serde_json::Value;

use crate::auth::Capability;
use crate::server::McpServer;

/// Required capability for all inject tools.
pub const REQUIRED: Capability = Capability::Inject;

/// Inject artifacts into Firefox/Chrome SQLite databases.
pub async fn inject_browser(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let browser = args
        .get("browser")
        .and_then(|v| v.as_str())
        .unwrap_or("firefox");
    let strategy = args
        .get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("direct");

    // Stub — real implementation uses plausiden-inject crate to modify browser DBs.
    Ok(serde_json::json!({
        "status": "stub",
        "browser": browser,
        "strategy": strategy,
        "message": "Stub — real implementation writes to browser SQLite databases",
        "warning": "This tool modifies real data on disk. Use with caution."
    }))
}

/// Inject files with realistic metadata onto the filesystem.
pub async fn inject_filesystem(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let target_dir = args
        .get("target_dir")
        .and_then(|v| v.as_str())
        .unwrap_or("/tmp");

    Ok(serde_json::json!({
        "status": "stub",
        "target_dir": target_dir,
        "message": "Stub — real implementation creates files with realistic MAC timestamps",
        "warning": "This tool creates files on disk. Use with caution."
    }))
}

/// Inject entries into system logs (journald/syslog).
pub async fn inject_logs(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let log_type = args
        .get("log_type")
        .and_then(|v| v.as_str())
        .unwrap_or("syslog");

    Ok(serde_json::json!({
        "status": "stub",
        "log_type": log_type,
        "message": "Stub — real implementation writes to journald or syslog",
        "warning": "This tool modifies system logs. Use with caution."
    }))
}
