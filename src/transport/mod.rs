//! Transport layer — stdio and optional HTTP+SSE transports.
//!
//! The stdio transport is the primary transport for CLI-based MCP clients
//! (Claude Code, Cursor). HTTP is optional and binds localhost only by default.

pub mod stdio;

#[cfg(feature = "http")]
pub mod http;
