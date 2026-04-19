//! Query capability — read-only access to status, stats, audit log.

use serde_json::Value;

use crate::auth::Capability;
use crate::server::McpServer;

/// Required capability.
pub const REQUIRED: Capability = Capability::Query;

/// Overall system status.
pub async fn status(_args: &Value, server: &McpServer) -> Result<Value, String> {
    let caps = server.capabilities.snapshot();
    let enabled: Vec<String> = caps
        .iter()
        .filter(|(_, v)| **v)
        .map(|(k, _)| k.to_string())
        .collect();

    Ok(serde_json::json!({
        "server": server.config.server_name,
        "version": server.config.server_version,
        "enabled_capabilities": enabled,
        "audit_entries": server.audit.len(),
        "limits": {
            "max_artifacts_per_minute": server.config.limits.max_artifacts_per_minute,
            "max_concurrent_tasks": server.config.limits.max_concurrent_tasks,
        }
    }))
}

/// Query the audit trail.
pub async fn audit_log(args: &Value, server: &McpServer) -> Result<Value, String> {
    let from_seq = args
        .get("from_sequence")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let entries = server.audit.entries_from(from_seq);
    let entries: Vec<_> = entries.into_iter().take(limit).collect();

    Ok(serde_json::json!({
        "entries": entries,
        "total": server.audit.len(),
        "chain_valid": server.audit.verify_chain().is_ok(),
    }))
}

/// Data categories ranked by forensic importance.
pub async fn forensic_weights(_args: &Value, _server: &McpServer) -> Result<Value, String> {
    Ok(serde_json::json!({
        "categories": [
            { "name": "browser_history", "weight": 0.95, "description": "Browsing history — most commonly used forensic evidence" },
            { "name": "search_queries", "weight": 0.92, "description": "Search engine queries — reveals intent" },
            { "name": "dns_queries", "weight": 0.88, "description": "DNS resolution logs — network-level evidence" },
            { "name": "file_metadata", "weight": 0.85, "description": "File MAC timestamps — filesystem forensics" },
            { "name": "cookies", "weight": 0.80, "description": "Browser cookies — session and tracking evidence" },
            { "name": "gps_traces", "weight": 0.78, "description": "Location data — physical presence evidence" },
            { "name": "contacts", "weight": 0.75, "description": "Contact lists — association evidence" },
            { "name": "system_logs", "weight": 0.70, "description": "OS event logs — activity timeline" },
            { "name": "network_traffic", "weight": 0.65, "description": "Traffic patterns — behavioral fingerprint" },
            { "name": "input_patterns", "weight": 0.50, "description": "Keystroke/touch dynamics — biometric evidence" },
        ]
    }))
}
