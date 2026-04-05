//! Server configuration — resource limits, transport settings, paths.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Resource limits to prevent runaway generation or injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum artifacts generated per minute.
    pub max_artifacts_per_minute: u32,
    /// Maximum bytes that can be injected to disk.
    pub max_injection_bytes: u64,
    /// Maximum swarm storage in bytes.
    pub max_swarm_storage_bytes: u64,
    /// Maximum swarm bandwidth per hour in bytes.
    pub max_swarm_bandwidth_per_hour: u64,
    /// Maximum concurrent background tasks.
    pub max_concurrent_tasks: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_artifacts_per_minute: 100,
            max_injection_bytes: 100 * 1024 * 1024,       // 100 MB
            max_swarm_storage_bytes: 500 * 1024 * 1024,    // 500 MB
            max_swarm_bandwidth_per_hour: 50 * 1024 * 1024, // 50 MB
            max_concurrent_tasks: 4,
        }
    }
}

/// Full server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Where to store profiles and audit log.
    pub data_dir: PathBuf,
    /// Resource limits.
    pub limits: ResourceLimits,
    /// Server name advertised in MCP initialization.
    pub server_name: String,
    /// Server version.
    pub server_version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let data_dir = dirs_default();
        Self {
            data_dir,
            limits: ResourceLimits::default(),
            server_name: "plausiden-mcp".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Default data directory: ~/.local/share/plausiden-mcp
fn dirs_default() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("plausiden-mcp")
    } else {
        PathBuf::from("/tmp/plausiden-mcp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.server_name, "plausiden-mcp");
        assert_eq!(config.limits.max_artifacts_per_minute, 100);
        assert_eq!(config.limits.max_concurrent_tasks, 4);
    }

    #[test]
    fn test_resource_limits_defaults() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_injection_bytes, 100 * 1024 * 1024);
        assert_eq!(limits.max_swarm_storage_bytes, 500 * 1024 * 1024);
    }
}
