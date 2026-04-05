//! MCP resources — read-only data exposed via plausiden:// URIs.

use serde_json::Value;

use crate::server::McpServer;

/// List all available resources.
pub fn list_all_resources() -> Vec<Value> {
    vec![
        serde_json::json!({
            "uri": "plausiden://profiles/presets",
            "name": "Profile Presets",
            "description": "Available profile presets with default settings",
            "mimeType": "application/json"
        }),
        serde_json::json!({
            "uri": "plausiden://categories",
            "name": "Data Categories",
            "description": "Data categories with forensic importance weights",
            "mimeType": "application/json"
        }),
        serde_json::json!({
            "uri": "plausiden://statistics",
            "name": "Generation Statistics",
            "description": "Cumulative generation statistics",
            "mimeType": "application/json"
        }),
        serde_json::json!({
            "uri": "plausiden://capabilities",
            "name": "Capability States",
            "description": "Current capability enabled/disabled states",
            "mimeType": "application/json"
        }),
    ]
}

/// Read a resource by URI.
pub fn read_resource(uri: &str, server: &McpServer) -> Result<Value, String> {
    match uri {
        "plausiden://profiles/presets" => Ok(serde_json::json!({
            "presets": [
                {
                    "name": "casual",
                    "description": "Average internet user — social media, news, shopping",
                    "categories": ["browser_history", "cookies", "searches"],
                    "intensity": "low"
                },
                {
                    "name": "researcher",
                    "description": "Academic/professional — papers, documentation, forums",
                    "categories": ["browser_history", "cookies", "searches", "file_metadata"],
                    "intensity": "medium"
                },
                {
                    "name": "journalist",
                    "description": "Journalist threat model — diverse sources, encrypted comms",
                    "categories": ["browser_history", "cookies", "searches", "contacts", "network_traffic"],
                    "intensity": "high"
                },
                {
                    "name": "activist",
                    "description": "Maximum protection — all categories, organic timing",
                    "categories": ["browser_history", "cookies", "searches", "file_metadata", "contacts", "gps_traces", "network_traffic", "system_logs"],
                    "intensity": "maximum"
                }
            ]
        })),

        "plausiden://categories" => Ok(serde_json::json!({
            "categories": [
                { "name": "browser_history", "forensic_weight": 0.95 },
                { "name": "search_queries", "forensic_weight": 0.92 },
                { "name": "dns_queries", "forensic_weight": 0.88 },
                { "name": "file_metadata", "forensic_weight": 0.85 },
                { "name": "cookies", "forensic_weight": 0.80 },
                { "name": "gps_traces", "forensic_weight": 0.78 },
                { "name": "contacts", "forensic_weight": 0.75 },
                { "name": "system_logs", "forensic_weight": 0.70 },
                { "name": "network_traffic", "forensic_weight": 0.65 },
                { "name": "input_patterns", "forensic_weight": 0.50 },
            ]
        })),

        "plausiden://statistics" => Ok(serde_json::json!({
            "total_artifacts_generated": 0,
            "total_injections": 0,
            "active_profile": "default",
            "uptime_seconds": 0,
            "note": "Stub — real implementation tracks cumulative stats"
        })),

        "plausiden://capabilities" => {
            let snap = server.capabilities.snapshot();
            let caps: Vec<Value> = snap
                .iter()
                .map(|(k, v)| serde_json::json!({ "name": k.to_string(), "enabled": v }))
                .collect();
            Ok(serde_json::json!({ "capabilities": caps }))
        }

        _ => Err(format!("Unknown resource URI: {uri}")),
    }
}
