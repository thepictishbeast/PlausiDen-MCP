//! Capability group modules — each maps to a set of tools.
//!
//! The six groups (Generate, Query, ProfileManagement, Schedule, Inject, Swarm)
//! are defined in `auth.rs`. This module provides the tool-to-capability mapping
//! and shared enable/disable tools that work across all groups.

pub mod generate;
pub mod inject;
pub mod swarm;
pub mod profile;
pub mod query;
pub mod system;
