> # ⚠️ DO NOT USE — UNVERIFIED — UNSAFE ⚠️
>
> This software is **unverified and unsafe for any production use**.
> It is published publicly only for transparency, third-party audit,
> and reproducibility. Treat every commit as guilty until proven
> innocent.
>
> By using this code you accept:
> - **No warranty** of any kind, express or implied.
> - **No fitness** for any particular purpose.
> - **No guarantee** of correctness, safety, or freedom from defects.
> - **Zero liability** on the maintainer for any damages — data loss,
>   security compromise, financial loss, or any consequential damages.
>
> The code is under active engineering development per the
> [Adversarial Validation Protocol v2](https://github.com/thepictishbeast/PlausiDen-AVP-Doctrine/blob/main/AVP2_PROTOCOL.md).
> Every commit's default verdict is **STILL BROKEN**. AVP-2 requires
> a minimum of 36 verification passes before a `SHIP-DECISION:`
> annotation may be considered. **No commit in this repository has
> reached `SHIP-DECISION:` status.**

# PlausiDen MCP Server

PlausiDen MCP turns the plausible deniability engine into a platform. Any MCP-compatible AI agent — Claude Code, Claude Desktop, Cursor, custom applications — can generate forensically realistic synthetic data, manage pollution profiles, and control the P2P swarm through a single, security-gated interface. It makes digital forensic evidence unreliable by giving every developer a standardized way to add plausible deniability to their tools.

## The Problem

Digital forensic evidence is treated as infallible by courts and law enforcement, but it is trivially manipulable and contextually meaningless. The presence of data on a device shifts the burden of proof to the defendant. Plausible deniability tools that generate indistinguishable synthetic data restore the correct burden of proof — but building these tools requires deep expertise in forensic artifacts, timing patterns, and data store formats. The MCP server abstracts this complexity behind a simple tool interface.

## How It Works

The MCP server exposes PlausiDen's capabilities through the [Model Context Protocol](https://modelcontextprotocol.io/), a standard JSON-RPC interface for AI agents. It provides **22 tools** across six capability groups, all gated by a capability-based access control system with a tamper-evident audit log.

### Security Model

**Everything is blocked by default.** Six capability groups must be explicitly enabled:

| Capability | Risk | Requires Acknowledgment |
|-----------|------|------------------------|
| Generate | Safe (memory only) | No |
| Query | Safe (read-only) | No |
| ProfileManagement | Low (config writes) | No |
| Schedule | Medium (background process) | No |
| Inject | **High (modifies real data)** | **Yes** |
| Swarm | **High (network access)** | **Yes** |

Every action is recorded in a tamper-evident audit log with a blake3 hash chain.

## Current Status

| Component | Status |
|-----------|--------|
| Capability-based access control | Done |
| Tamper-evident audit log | Done |
| MCP JSON-RPC server | Done |
| 22 tool definitions + 3 system tools | Done |
| 4 MCP resources | Done |
| 3 workflow prompts | Done |
| Stdio transport | Done |
| 43 tests passing | Done |
| Engine integration | Stub (awaiting plausiden-engine) |
| Inject integration | Stub (awaiting plausiden-inject) |
| Swarm integration | Stub (awaiting plausiden-swarm) |
| HTTP transport | Planned |

## Quick Start

```bash
# Build
cargo build --release

# Run on stdio (for Claude Code / Cursor)
./target/release/plausiden-mcp

# Add to Claude Code:
claude mcp add plausiden --scope user -- /path/to/plausiden-mcp
```

## Available Tools

### System (always available)
| Tool | Description |
|------|-------------|
| `plausiden_enable_capability` | Enable a capability group |
| `plausiden_disable_capability` | Disable a capability group |
| `plausiden_list_capabilities` | List all capabilities with state |

### Generate (requires `generate` capability)
| Tool | Description |
|------|-------------|
| `plausiden_generate_browser_history` | Browser history with referrer chains |
| `plausiden_generate_cookies` | Cookies matching browsing profile |
| `plausiden_generate_searches` | Search queries with topic drift |
| `plausiden_generate_files` | File metadata with realistic timestamps |
| `plausiden_generate_contacts` | Contacts with locale-appropriate names |
| `plausiden_generate_location` | GPS traces on real road networks |
| `plausiden_generate_network` | DNS queries, HTTP timing, TLS fingerprints |

### Inject (requires acknowledgment)
| Tool | Description |
|------|-------------|
| `plausiden_inject_browser` | Write to Firefox/Chrome databases |
| `plausiden_inject_filesystem` | Create files with realistic metadata |
| `plausiden_inject_logs` | Write to journald/syslog |

### Swarm (requires acknowledgment)
| Tool | Description |
|------|-------------|
| `plausiden_swarm_join` | Connect to P2P network |
| `plausiden_swarm_leave` | Disconnect from network |
| `plausiden_swarm_status` | Query connection status |

### Query (requires `query` capability)
| Tool | Description |
|------|-------------|
| `plausiden_status` | System overview |
| `plausiden_audit_log` | Query audit trail |
| `plausiden_forensic_weights` | Data categories by forensic importance |

## The PlausiDen Ecosystem

This repo is the MCP integration layer for the PlausiDen protection suite. Related repos:
- **plausiden-engine** — Core data generation engine
- **plausiden-inject** — OS data store injection
- **plausiden-swarm** — P2P swarm network
- **PlausiDenOS** — Sovereign mobile OS with seL4 protection domains

## License

Apache-2.0 — maximizes adoption as an integration layer.
