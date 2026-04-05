//! Swarm capability — NETWORK ACCESS: connects to the P2P network.
//!
//! All swarm tools require the Swarm capability, which demands typed
//! acknowledgment before enabling. Network connections are audit-logged.

use serde_json::Value;

use crate::auth::Capability;
use crate::server::McpServer;

/// Required capability for all swarm tools.
pub const REQUIRED: Capability = Capability::Swarm;

/// Join the P2P swarm network.
pub async fn swarm_join(args: &Value, _server: &McpServer) -> Result<Value, String> {
    let max_peers = args.get("max_peers").and_then(|v| v.as_u64()).unwrap_or(50);
    let max_storage_mb = args.get("max_storage_mb").and_then(|v| v.as_u64()).unwrap_or(500);

    Ok(serde_json::json!({
        "status": "stub",
        "max_peers": max_peers,
        "max_storage_mb": max_storage_mb,
        "message": "Stub — real implementation connects via plausiden-swarm (iroh-based)"
    }))
}

/// Leave the P2P swarm network.
pub async fn swarm_leave(_args: &Value, _server: &McpServer) -> Result<Value, String> {
    Ok(serde_json::json!({
        "status": "stub",
        "message": "Stub — disconnects from swarm, frees resources"
    }))
}

/// Query swarm connection status.
pub async fn swarm_status(_args: &Value, _server: &McpServer) -> Result<Value, String> {
    Ok(serde_json::json!({
        "connected": false,
        "peer_count": 0,
        "fragments_held": 0,
        "bandwidth_used_bytes": 0,
        "storage_used_bytes": 0,
        "message": "Stub — real implementation queries live swarm state"
    }))
}
