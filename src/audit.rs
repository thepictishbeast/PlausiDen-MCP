//! Tamper-evident audit log with blake3 hash chain.
//!
//! Every action taken through the MCP server is recorded here. Each entry
//! includes a hash of the previous entry, forming a chain that makes
//! retroactive tampering detectable.

use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::Capability;

/// Types of events recorded in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    /// Server started.
    ServerStart,
    /// Server stopped.
    ServerStop,
    /// A capability was enabled.
    CapabilityEnabled { capability: Capability },
    /// A capability was disabled.
    CapabilityDisabled { capability: Capability },
    /// A tool was called successfully.
    ToolCalled {
        tool_name: String,
        /// blake3 hash of the serialized parameters (not the params themselves).
        param_hash: String,
    },
    /// A tool call was denied due to missing capability.
    ToolDenied {
        tool_name: String,
        required_capability: Capability,
    },
    /// Server configuration changed.
    ConfigChanged { key: String },
    /// An error occurred.
    Error { message: String },
}

/// A single entry in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Monotonic sequence number.
    pub sequence: u64,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// The event that occurred.
    pub event: AuditEvent,
    /// blake3 hash of the previous entry (hex-encoded). Empty string for the first entry.
    pub prev_hash: String,
    /// blake3 hash of this entry (hex-encoded).
    pub hash: String,
}

impl AuditEntry {
    /// Compute the hash for this entry (excluding the hash field itself).
    fn compute_hash(sequence: u64, timestamp: &DateTime<Utc>, event: &AuditEvent, prev_hash: &str) -> String {
        let payload = format!(
            "{}:{}:{}:{}",
            sequence,
            timestamp.to_rfc3339(),
            serde_json::to_string(event).unwrap_or_default(),
            prev_hash,
        );
        blake3::hash(payload.as_bytes()).to_hex().to_string()
    }
}

/// Thread-safe append-only audit log.
#[derive(Debug, Clone)]
pub struct AuditLog {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record an event in the audit log.
    pub fn record(&self, event: AuditEvent) -> AuditEntry {
        let mut entries = self.entries.write().expect("audit lock poisoned");
        let sequence = entries.len() as u64;
        let timestamp = Utc::now();
        let prev_hash = entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_default();

        let hash = AuditEntry::compute_hash(sequence, &timestamp, &event, &prev_hash);

        let entry = AuditEntry {
            sequence,
            timestamp,
            event,
            prev_hash,
            hash,
        };

        let cloned = entry.clone();
        entries.push(entry);
        cloned
    }

    /// Get all entries in the log.
    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries.read().expect("audit lock poisoned").clone()
    }

    /// Get entries starting from a given sequence number.
    pub fn entries_from(&self, from_seq: u64) -> Vec<AuditEntry> {
        let entries = self.entries.read().expect("audit lock poisoned");
        entries
            .iter()
            .filter(|e| e.sequence >= from_seq)
            .cloned()
            .collect()
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.read().expect("audit lock poisoned").len()
    }

    /// Whether the audit log is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Verify the integrity of the hash chain. Returns Ok(()) if valid,
    /// or Err with the sequence number of the first broken link.
    pub fn verify_chain(&self) -> Result<(), u64> {
        let entries = self.entries.read().expect("audit lock poisoned");

        for (i, entry) in entries.iter().enumerate() {
            // Verify the previous hash links correctly.
            if i == 0 {
                if !entry.prev_hash.is_empty() {
                    return Err(0);
                }
            } else if entry.prev_hash != entries[i - 1].hash {
                return Err(entry.sequence);
            }

            // Verify the entry's own hash.
            let expected =
                AuditEntry::compute_hash(entry.sequence, &entry.timestamp, &entry.event, &entry.prev_hash);
            if entry.hash != expected {
                return Err(entry.sequence);
            }
        }

        Ok(())
    }

    /// Hash arbitrary parameters for the ToolCalled event.
    pub fn hash_params(params: &serde_json::Value) -> String {
        let serialized = serde_json::to_string(params).unwrap_or_default();
        blake3::hash(serialized.as_bytes()).to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_log() {
        let log = AuditLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_record_and_retrieve() {
        let log = AuditLog::new();
        log.record(AuditEvent::ServerStart);
        assert_eq!(log.len(), 1);

        let entries = log.entries();
        assert_eq!(entries[0].sequence, 0);
        assert!(entries[0].prev_hash.is_empty());
        assert!(!entries[0].hash.is_empty());
    }

    #[test]
    fn test_hash_chain_links() {
        let log = AuditLog::new();
        log.record(AuditEvent::ServerStart);
        log.record(AuditEvent::CapabilityEnabled {
            capability: Capability::Generate,
        });
        log.record(AuditEvent::CapabilityEnabled {
            capability: Capability::Query,
        });

        let entries = log.entries();
        assert!(entries[0].prev_hash.is_empty());
        assert_eq!(entries[1].prev_hash, entries[0].hash);
        assert_eq!(entries[2].prev_hash, entries[1].hash);
    }

    #[test]
    fn test_verify_chain_valid() {
        let log = AuditLog::new();
        log.record(AuditEvent::ServerStart);
        log.record(AuditEvent::CapabilityEnabled {
            capability: Capability::Generate,
        });
        log.record(AuditEvent::ToolCalled {
            tool_name: "test_tool".to_string(),
            param_hash: "abc123".to_string(),
        });
        log.record(AuditEvent::ServerStop);

        assert!(log.verify_chain().is_ok());
    }

    #[test]
    fn test_capability_change_logged() {
        let log = AuditLog::new();
        log.record(AuditEvent::CapabilityEnabled {
            capability: Capability::Inject,
        });
        log.record(AuditEvent::CapabilityDisabled {
            capability: Capability::Inject,
        });

        let entries = log.entries();
        assert_eq!(entries.len(), 2);
        assert!(matches!(
            &entries[0].event,
            AuditEvent::CapabilityEnabled { capability: Capability::Inject }
        ));
        assert!(matches!(
            &entries[1].event,
            AuditEvent::CapabilityDisabled { capability: Capability::Inject }
        ));
    }

    #[test]
    fn test_tool_call_logged() {
        let log = AuditLog::new();
        let params = serde_json::json!({"count": 10, "profile": "journalist"});
        let param_hash = AuditLog::hash_params(&params);

        log.record(AuditEvent::ToolCalled {
            tool_name: "plausiden_generate_browser_history".to_string(),
            param_hash: param_hash.clone(),
        });

        let entries = log.entries();
        assert!(matches!(
            &entries[0].event,
            AuditEvent::ToolCalled { tool_name, param_hash: ph }
                if tool_name == "plausiden_generate_browser_history" && *ph == param_hash
        ));
    }

    #[test]
    fn test_denied_call_logged() {
        let log = AuditLog::new();
        log.record(AuditEvent::ToolDenied {
            tool_name: "plausiden_inject_browser".to_string(),
            required_capability: Capability::Inject,
        });

        let entries = log.entries();
        assert!(matches!(
            &entries[0].event,
            AuditEvent::ToolDenied {
                tool_name,
                required_capability: Capability::Inject
            } if tool_name == "plausiden_inject_browser"
        ));
    }

    #[test]
    fn test_entries_from() {
        let log = AuditLog::new();
        log.record(AuditEvent::ServerStart);
        log.record(AuditEvent::CapabilityEnabled {
            capability: Capability::Generate,
        });
        log.record(AuditEvent::ServerStop);

        let from_1 = log.entries_from(1);
        assert_eq!(from_1.len(), 2);
        assert_eq!(from_1[0].sequence, 1);
    }

    #[test]
    fn test_hash_params_deterministic() {
        let params = serde_json::json!({"count": 10});
        let h1 = AuditLog::hash_params(&params);
        let h2 = AuditLog::hash_params(&params);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_params_different_for_different_input() {
        let p1 = serde_json::json!({"count": 10});
        let p2 = serde_json::json!({"count": 20});
        assert_ne!(AuditLog::hash_params(&p1), AuditLog::hash_params(&p2));
    }
}
