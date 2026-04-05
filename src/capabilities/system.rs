//! System capability tools — enable/disable capabilities, configuration.
//!
//! These tools do NOT require any capability to be enabled (they manage
//! capabilities themselves). They are always available.

use serde_json::Value;

use crate::audit::AuditEvent;
use crate::auth::Capability;
use crate::server::McpServer;

/// Enable a capability. Safe capabilities need no acknowledgment.
/// Inject and Swarm require the typed acknowledgment string.
pub async fn enable_capability(args: &Value, server: &McpServer) -> Result<Value, String> {
    let cap_name = args
        .get("capability")
        .and_then(|v| v.as_str())
        .ok_or("missing 'capability' parameter")?;

    let ack = args.get("acknowledgment").and_then(|v| v.as_str());

    let cap = parse_capability(cap_name)?;

    server
        .capabilities
        .enable(cap, ack)
        .map_err(|e| e.to_string())?;

    server.audit.record(AuditEvent::CapabilityEnabled {
        capability: cap,
    });

    Ok(serde_json::json!({
        "status": "enabled",
        "capability": cap_name,
        "description": cap.description(),
    }))
}

/// Disable a capability.
pub async fn disable_capability(args: &Value, server: &McpServer) -> Result<Value, String> {
    let cap_name = args
        .get("capability")
        .and_then(|v| v.as_str())
        .ok_or("missing 'capability' parameter")?;

    let cap = parse_capability(cap_name)?;

    server.capabilities.disable(cap);

    server.audit.record(AuditEvent::CapabilityDisabled {
        capability: cap,
    });

    Ok(serde_json::json!({
        "status": "disabled",
        "capability": cap_name,
    }))
}

/// List all capabilities with their current state.
pub async fn list_capabilities(_args: &Value, server: &McpServer) -> Result<Value, String> {
    let snap = server.capabilities.snapshot();
    let caps: Vec<Value> = Capability::all()
        .iter()
        .map(|cap| {
            serde_json::json!({
                "name": cap.to_string(),
                "enabled": snap.get(cap).copied().unwrap_or(false),
                "requires_acknowledgment": cap.requires_acknowledgment(),
                "description": cap.description(),
            })
        })
        .collect();

    Ok(serde_json::json!({ "capabilities": caps }))
}

/// Parse a capability name string into the enum.
fn parse_capability(name: &str) -> Result<Capability, String> {
    match name {
        "generate" => Ok(Capability::Generate),
        "query" => Ok(Capability::Query),
        "profile_management" => Ok(Capability::ProfileManagement),
        "schedule" => Ok(Capability::Schedule),
        "inject" => Ok(Capability::Inject),
        "swarm" => Ok(Capability::Swarm),
        _ => Err(format!(
            "Unknown capability '{}'. Valid: generate, query, profile_management, schedule, inject, swarm",
            name
        )),
    }
}
