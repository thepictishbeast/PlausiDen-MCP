//! Capability-based access control for the MCP server.
//!
//! All capabilities are disabled by default. Inject and Swarm require typed
//! acknowledgment strings before they can be enabled. Every capability state
//! change is recorded in the audit log.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The six capability groups. All are off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Generate data in memory (no disk, no network) — safest.
    Generate,
    /// Read-only queries — status, stats, audit log.
    Query,
    /// Profile management — writes config file only.
    ProfileManagement,
    /// Background scheduling — persistent process, uses resources.
    Schedule,
    /// Inject into OS data stores — MODIFIES REAL DATA.
    /// Requires acknowledgment: "I understand this will modify real data on disk"
    Inject,
    /// P2P swarm — NETWORK ACCESS.
    /// Requires acknowledgment: "I understand this will open network connections"
    Swarm,
}

impl Capability {
    /// Returns all capability variants.
    pub fn all() -> &'static [Capability] {
        &[
            Capability::Generate,
            Capability::Query,
            Capability::ProfileManagement,
            Capability::Schedule,
            Capability::Inject,
            Capability::Swarm,
        ]
    }

    /// Whether this capability requires typed acknowledgment to enable.
    pub fn requires_acknowledgment(&self) -> bool {
        matches!(self, Capability::Inject | Capability::Swarm)
    }

    /// The acknowledgment string required to enable dangerous capabilities.
    pub fn acknowledgment_string(&self) -> Option<&'static str> {
        match self {
            Capability::Inject => {
                Some("I understand this will modify real data on disk")
            }
            Capability::Swarm => {
                Some("I understand this will open network connections")
            }
            _ => None,
        }
    }

    /// Human-readable description of what this capability grants.
    pub fn description(&self) -> &'static str {
        match self {
            Capability::Generate => "Generate synthetic data in memory (no disk writes, no network)",
            Capability::Query => "Read-only queries: status, statistics, audit log",
            Capability::ProfileManagement => "Create, list, and switch user profiles (writes config only)",
            Capability::Schedule => "Start/stop background generation (persistent process)",
            Capability::Inject => "Inject artifacts into OS data stores (MODIFIES REAL DATA)",
            Capability::Swarm => "Join/leave P2P swarm network (OPENS NETWORK CONNECTIONS)",
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Generate => write!(f, "generate"),
            Capability::Query => write!(f, "query"),
            Capability::ProfileManagement => write!(f, "profile_management"),
            Capability::Schedule => write!(f, "schedule"),
            Capability::Inject => write!(f, "inject"),
            Capability::Swarm => write!(f, "swarm"),
        }
    }
}

/// Errors from the capability system.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("capability '{0}' is not enabled")]
    CapabilityDisabled(Capability),

    #[error("capability '{0}' requires acknowledgment: \"{1}\"")]
    AcknowledgmentRequired(Capability, &'static str),

    #[error("invalid acknowledgment for capability '{0}': expected \"{1}\"")]
    InvalidAcknowledgment(Capability, &'static str),
}

/// Thread-safe capability store. Tracks which capabilities are currently enabled.
#[derive(Debug, Clone)]
pub struct CapabilityStore {
    states: Arc<RwLock<HashMap<Capability, bool>>>,
}

impl Default for CapabilityStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityStore {
    /// Create a new store with all capabilities disabled.
    pub fn new() -> Self {
        let mut states = HashMap::new();
        for cap in Capability::all() {
            states.insert(*cap, false);
        }
        Self {
            states: Arc::new(RwLock::new(states)),
        }
    }

    /// Check whether a capability is currently enabled.
    pub fn is_enabled(&self, cap: Capability) -> bool {
        self.states
            .read()
            .expect("capability lock poisoned")
            .get(&cap)
            .copied()
            .unwrap_or(false)
    }

    /// Require that a capability is enabled, returning an error if not.
    pub fn require(&self, cap: Capability) -> Result<(), AuthError> {
        if self.is_enabled(cap) {
            Ok(())
        } else {
            Err(AuthError::CapabilityDisabled(cap))
        }
    }

    /// Enable a capability. For Inject and Swarm, the caller must provide
    /// the correct acknowledgment string.
    pub fn enable(
        &self,
        cap: Capability,
        acknowledgment: Option<&str>,
    ) -> Result<(), AuthError> {
        if cap.requires_acknowledgment() {
            let required = cap
                .acknowledgment_string()
                .expect("dangerous caps always have an ack string");
            match acknowledgment {
                None => return Err(AuthError::AcknowledgmentRequired(cap, required)),
                Some(ack) if ack != required => {
                    return Err(AuthError::InvalidAcknowledgment(cap, required));
                }
                Some(_) => {}
            }
        }
        self.states
            .write()
            .expect("capability lock poisoned")
            .insert(cap, true);
        Ok(())
    }

    /// Disable a capability.
    pub fn disable(&self, cap: Capability) {
        self.states
            .write()
            .expect("capability lock poisoned")
            .insert(cap, false);
    }

    /// Get a snapshot of all capability states.
    pub fn snapshot(&self) -> HashMap<Capability, bool> {
        self.states
            .read()
            .expect("capability lock poisoned")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_capabilities_disabled_by_default() {
        let store = CapabilityStore::new();
        for cap in Capability::all() {
            assert!(
                !store.is_enabled(*cap),
                "{cap} should be disabled by default"
            );
        }
    }

    #[test]
    fn test_enable_safe_capability() {
        let store = CapabilityStore::new();
        store.enable(Capability::Generate, None).unwrap();
        assert!(store.is_enabled(Capability::Generate));
    }

    #[test]
    fn test_enable_query_no_ack_needed() {
        let store = CapabilityStore::new();
        store.enable(Capability::Query, None).unwrap();
        assert!(store.is_enabled(Capability::Query));
    }

    #[test]
    fn test_inject_requires_acknowledgment() {
        let store = CapabilityStore::new();
        let err = store.enable(Capability::Inject, None).unwrap_err();
        assert!(matches!(err, AuthError::AcknowledgmentRequired(Capability::Inject, _)));
    }

    #[test]
    fn test_inject_wrong_acknowledgment() {
        let store = CapabilityStore::new();
        let err = store
            .enable(Capability::Inject, Some("wrong"))
            .unwrap_err();
        assert!(matches!(err, AuthError::InvalidAcknowledgment(Capability::Inject, _)));
    }

    #[test]
    fn test_inject_correct_acknowledgment() {
        let store = CapabilityStore::new();
        store
            .enable(
                Capability::Inject,
                Some("I understand this will modify real data on disk"),
            )
            .unwrap();
        assert!(store.is_enabled(Capability::Inject));
    }

    #[test]
    fn test_swarm_requires_acknowledgment() {
        let store = CapabilityStore::new();
        let err = store.enable(Capability::Swarm, None).unwrap_err();
        assert!(matches!(err, AuthError::AcknowledgmentRequired(Capability::Swarm, _)));
    }

    #[test]
    fn test_swarm_correct_acknowledgment() {
        let store = CapabilityStore::new();
        store
            .enable(
                Capability::Swarm,
                Some("I understand this will open network connections"),
            )
            .unwrap();
        assert!(store.is_enabled(Capability::Swarm));
    }

    #[test]
    fn test_disable_capability() {
        let store = CapabilityStore::new();
        store.enable(Capability::Generate, None).unwrap();
        assert!(store.is_enabled(Capability::Generate));
        store.disable(Capability::Generate);
        assert!(!store.is_enabled(Capability::Generate));
    }

    #[test]
    fn test_require_enabled() {
        let store = CapabilityStore::new();
        store.enable(Capability::Query, None).unwrap();
        assert!(store.require(Capability::Query).is_ok());
    }

    #[test]
    fn test_require_disabled() {
        let store = CapabilityStore::new();
        assert!(store.require(Capability::Generate).is_err());
    }

    #[test]
    fn test_snapshot() {
        let store = CapabilityStore::new();
        store.enable(Capability::Generate, None).unwrap();
        store.enable(Capability::Query, None).unwrap();
        let snap = store.snapshot();
        assert_eq!(snap[&Capability::Generate], true);
        assert_eq!(snap[&Capability::Query], true);
        assert_eq!(snap[&Capability::Inject], false);
        assert_eq!(snap[&Capability::Swarm], false);
    }
}
