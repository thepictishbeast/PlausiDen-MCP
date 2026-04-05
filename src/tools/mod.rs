//! Tool registry — maps tool names to capability requirements and handlers.

use serde_json::Value;

use crate::auth::Capability;
use crate::capabilities::{generate, inject, swarm, profile, query, system};
use crate::server::McpServer;

/// Get the required capability for a tool, or None if the tool is always available.
pub fn required_capability(tool_name: &str) -> Option<Capability> {
    match tool_name {
        // System tools — always available (manage capabilities themselves).
        "plausiden_enable_capability"
        | "plausiden_disable_capability"
        | "plausiden_list_capabilities" => None,

        // Generate tools.
        "plausiden_generate_browser_history"
        | "plausiden_generate_cookies"
        | "plausiden_generate_searches"
        | "plausiden_generate_files"
        | "plausiden_generate_contacts"
        | "plausiden_generate_location"
        | "plausiden_generate_network" => Some(generate::REQUIRED),

        // Inject tools.
        "plausiden_inject_browser"
        | "plausiden_inject_filesystem"
        | "plausiden_inject_logs" => Some(inject::REQUIRED),

        // Swarm tools.
        "plausiden_swarm_join"
        | "plausiden_swarm_leave"
        | "plausiden_swarm_status" => Some(swarm::REQUIRED),

        // Profile tools.
        "plausiden_profile_create"
        | "plausiden_profile_list"
        | "plausiden_profile_switch" => Some(profile::REQUIRED),

        // Query tools.
        "plausiden_status"
        | "plausiden_audit_log"
        | "plausiden_forensic_weights" => Some(query::REQUIRED),

        // Unknown tool.
        _ => None,
    }
}

/// Dispatch a tool call to its handler.
pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    server: &McpServer,
) -> Result<Value, String> {
    match tool_name {
        // System.
        "plausiden_enable_capability" => system::enable_capability(args, server).await,
        "plausiden_disable_capability" => system::disable_capability(args, server).await,
        "plausiden_list_capabilities" => system::list_capabilities(args, server).await,

        // Generate.
        "plausiden_generate_browser_history" => generate::generate_browser_history(args, server).await,
        "plausiden_generate_cookies" => generate::generate_cookies(args, server).await,
        "plausiden_generate_searches" => generate::generate_searches(args, server).await,
        "plausiden_generate_files" => generate::generate_files(args, server).await,
        "plausiden_generate_contacts" => generate::generate_contacts(args, server).await,
        "plausiden_generate_location" => generate::generate_location(args, server).await,
        "plausiden_generate_network" => generate::generate_network(args, server).await,

        // Inject.
        "plausiden_inject_browser" => inject::inject_browser(args, server).await,
        "plausiden_inject_filesystem" => inject::inject_filesystem(args, server).await,
        "plausiden_inject_logs" => inject::inject_logs(args, server).await,

        // Swarm.
        "plausiden_swarm_join" => swarm::swarm_join(args, server).await,
        "plausiden_swarm_leave" => swarm::swarm_leave(args, server).await,
        "plausiden_swarm_status" => swarm::swarm_status(args, server).await,

        // Profile.
        "plausiden_profile_create" => profile::profile_create(args, server).await,
        "plausiden_profile_list" => profile::profile_list(args, server).await,
        "plausiden_profile_switch" => profile::profile_switch(args, server).await,

        // Query.
        "plausiden_status" => query::status(args, server).await,
        "plausiden_audit_log" => query::audit_log(args, server).await,
        "plausiden_forensic_weights" => query::forensic_weights(args, server).await,

        _ => Err(format!("Unknown tool: {tool_name}")),
    }
}

/// Return the full list of tool definitions for tools/list.
pub fn list_all_tools() -> Vec<Value> {
    vec![
        // System (always available).
        tool_def(
            "plausiden_enable_capability",
            "Enable a capability group. Inject and Swarm require typed acknowledgment.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capability": {
                        "type": "string",
                        "enum": ["generate", "query", "profile_management", "schedule", "inject", "swarm"],
                        "description": "The capability to enable"
                    },
                    "acknowledgment": {
                        "type": "string",
                        "description": "Required for inject and swarm capabilities"
                    }
                },
                "required": ["capability"]
            }),
        ),
        tool_def(
            "plausiden_disable_capability",
            "Disable a capability group.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capability": {
                        "type": "string",
                        "enum": ["generate", "query", "profile_management", "schedule", "inject", "swarm"]
                    }
                },
                "required": ["capability"]
            }),
        ),
        tool_def(
            "plausiden_list_capabilities",
            "List all capabilities and their current enabled/disabled state.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),

        // Generate tools.
        tool_def(
            "plausiden_generate_browser_history",
            "Generate browser history entries with referrer chains. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_cookies",
            "Generate cookie artifacts matching a browsing profile. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_searches",
            "Generate search query artifacts with natural topic drift. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_files",
            "Generate filesystem artifact metadata with realistic timestamps. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_contacts",
            "Generate contact and call log artifacts. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_location",
            "Generate GPS traces anchored to real road networks. Requires: generate capability.",
            count_profile_schema(),
        ),
        tool_def(
            "plausiden_generate_network",
            "Generate DNS queries, HTTP timing, TLS fingerprints. Requires: generate capability.",
            count_profile_schema(),
        ),

        // Inject tools.
        tool_def(
            "plausiden_inject_browser",
            "Inject artifacts into Firefox/Chrome SQLite databases. DANGEROUS: modifies real data. Requires: inject capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "browser": { "type": "string", "enum": ["firefox", "chrome"], "default": "firefox" },
                    "strategy": { "type": "string", "enum": ["direct", "translator"], "default": "direct" }
                }
            }),
        ),
        tool_def(
            "plausiden_inject_filesystem",
            "Create files with realistic metadata on disk. DANGEROUS. Requires: inject capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "target_dir": { "type": "string", "description": "Directory to inject files into" }
                }
            }),
        ),
        tool_def(
            "plausiden_inject_logs",
            "Write entries to system logs. DANGEROUS. Requires: inject capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "log_type": { "type": "string", "enum": ["syslog", "journald"], "default": "syslog" }
                }
            }),
        ),

        // Swarm tools.
        tool_def(
            "plausiden_swarm_join",
            "Join the P2P swarm network. Requires: swarm capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "max_peers": { "type": "integer", "default": 50 },
                    "max_storage_mb": { "type": "integer", "default": 500 }
                }
            }),
        ),
        tool_def(
            "plausiden_swarm_leave",
            "Leave the P2P swarm network. Requires: swarm capability.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "plausiden_swarm_status",
            "Query P2P swarm connection status. Requires: swarm capability.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),

        // Profile tools.
        tool_def(
            "plausiden_profile_create",
            "Create a new user profile from a preset. Requires: profile_management capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Profile name" },
                    "preset": { "type": "string", "enum": ["casual", "researcher", "journalist", "activist"], "default": "casual" }
                },
                "required": ["name"]
            }),
        ),
        tool_def(
            "plausiden_profile_list",
            "List all user profiles. Requires: profile_management capability.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "plausiden_profile_switch",
            "Switch the active profile. Requires: profile_management capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Profile to activate" }
                },
                "required": ["name"]
            }),
        ),

        // Query tools.
        tool_def(
            "plausiden_status",
            "Get overall system status. Requires: query capability.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "plausiden_audit_log",
            "Query the tamper-evident audit trail. Requires: query capability.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "from_sequence": { "type": "integer", "default": 0 },
                    "limit": { "type": "integer", "default": 50 }
                }
            }),
        ),
        tool_def(
            "plausiden_forensic_weights",
            "Get data categories ranked by forensic importance. Requires: query capability.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
    ]
}

/// Helper to create a tool definition JSON object.
fn tool_def(name: &str, description: &str, input_schema: Value) -> Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

/// Common schema for generate tools (count + profile).
fn count_profile_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer", "description": "Number of artifacts to generate", "default": 10 },
            "profile": { "type": "string", "enum": ["casual", "researcher", "journalist", "activist"], "default": "casual" }
        }
    })
}
