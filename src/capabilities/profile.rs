//! Profile management capability — create, list, and switch user profiles.

use serde_json::Value;

use crate::auth::Capability;
use crate::server::McpServer;

/// Required capability.
pub const REQUIRED: Capability = Capability::ProfileManagement;

/// Create a new profile from a preset base.
pub async fn profile_create(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("default");
    let preset = args.get("preset").and_then(|v| v.as_str()).unwrap_or("casual");

    Ok(serde_json::json!({
        "status": "created",
        "profile": {
            "name": name,
            "preset": preset,
            "categories": ["browser_history", "cookies", "searches"],
            "intensity": "medium"
        }
    }))
}

/// List all profiles.
pub async fn profile_list(_args: &Value, _server: &McpServer) -> Result<Value, String> {
    Ok(serde_json::json!({
        "profiles": [
            { "name": "default", "preset": "casual", "active": true },
        ],
        "note": "Stub — profiles stored in data_dir/profiles/"
    }))
}

/// Switch the active profile.
pub async fn profile_switch(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("default");

    Ok(serde_json::json!({
        "status": "switched",
        "active_profile": name,
    }))
}
