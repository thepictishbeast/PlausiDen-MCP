//! PlausiDen MCP Server — exposes the plausible deniability engine to any MCP client.
//!
//! This crate implements a Model Context Protocol server that lets Claude Code,
//! Claude Desktop, Cursor, and custom applications generate forensically realistic
//! synthetic data, manage pollution profiles, and optionally inject artifacts into
//! OS data stores — all through MCP's standard tool/resource/prompt primitives.
//!
//! # Security Model
//!
//! Everything is blocked by default. Six capability groups must be explicitly
//! enabled before any tool becomes available. The two most dangerous capabilities
//! (Inject and Swarm) require typed acknowledgment. Every action is recorded in
//! a tamper-evident audit log with a blake3 hash chain.
//!
//! # Example
//!
//! ```rust,no_run
//! use plausiden_mcp::server::McpServer;
//! use plausiden_mcp::config::ServerConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig::default();
//!     let server = McpServer::new(config);
//!     server.run_stdio().await.expect("server failed");
//! }
//! ```

pub mod auth;
pub mod audit;
pub mod config;
pub mod server;
pub mod capabilities;
pub mod tools;
pub mod resources;
pub mod prompts;
pub mod transport;
