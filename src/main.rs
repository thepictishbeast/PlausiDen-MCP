//! PlausiDen MCP Server
//!
//! Model Context Protocol server providing AI agents with structured access
//! to the PlausiDen ecosystem: engine generation, injection targets, swarm
//! status, and shield orchestration.
//!
//! ## Protocol
//!
//! This server communicates over stdin/stdout using the MCP JSON-RPC protocol.
//! It exposes PlausiDen capabilities as MCP tools that Claude Code and other
//! AI agents can invoke.

mod protocol;
mod tools;

use clap::Parser;
use tracing_subscriber::EnvFilter;

/// PlausiDen MCP Server — AI agent interface to the PlausiDen ecosystem.
#[derive(Parser, Debug)]
#[command(name = "plausiden-mcp", version, about)]
struct Args {
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&args.log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("PlausiDen MCP server starting");

    let server = protocol::McpServer::new();
    server.run().await
}
