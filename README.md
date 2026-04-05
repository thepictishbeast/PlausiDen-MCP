# PlausiDen MCP Server

Model Context Protocol server that gives AI agents structured access to the PlausiDen ecosystem. Generate synthetic artifacts, inject them into OS data stores, monitor swarm status, and assess threats — all through MCP tool calls.

## Quick Start

```bash
cargo build --release
# Add to Claude Code settings:
# "mcpServers": { "plausiden": { "command": "./target/release/plausiden-mcp" } }
```

## Available Tools

| Tool | Description |
|------|-------------|
| `plausiden_generate` | Generate synthetic artifacts (history, cookies, files, etc.) |
| `plausiden_list_generators` | List all generators with forensic weight and status |
| `plausiden_configure_profile` | Set user profile for generation |
| `plausiden_list_targets` | List injection targets on this system |
| `plausiden_inject` | Inject artifacts into OS data stores |
| `plausiden_swarm_status` | P2P swarm node status |
| `plausiden_threat_assessment` | Detect forensic tools, evaluate posture |
| `plausiden_ecosystem_status` | All repo build/test status |

## The PlausiDen Ecosystem

Part of PlausiDen — an AI-powered plausible deniability suite. See the full ecosystem overview.

## License

BSL 1.1 with Apache 2.0 change date of 2030-04-04.
