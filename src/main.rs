//! PlausiDen MCP Server — entry point.
//!
//! Runs the MCP server on stdio by default. Use `--transport http` for HTTP.

use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use plausiden_mcp::config::ServerConfig;
use plausiden_mcp::server::McpServer;

#[derive(Parser)]
#[command(name = "plausiden-mcp", version, about = "PlausiDen MCP Server — plausible deniability engine for any MCP client")]
struct Cli {
    /// Transport to use: stdio or http.
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// Data directory for profiles and audit log.
    #[arg(long)]
    data_dir: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing (logs to stderr so stdout stays clean for JSON-RPC).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let mut config = ServerConfig::default();
    if let Some(data_dir) = cli.data_dir {
        config.data_dir = data_dir;
    }

    let server = Arc::new(McpServer::new(config));

    match cli.transport.as_str() {
        "stdio" => {
            plausiden_mcp::transport::stdio::run(server).await?;
        }
        #[cfg(feature = "http")]
        "http" => {
            plausiden_mcp::transport::http::run(server).await?;
        }
        other => {
            eprintln!("Unknown transport: {other}. Use 'stdio' or 'http' (with --features http).");
            std::process::exit(1);
        }
    }

    Ok(())
}
