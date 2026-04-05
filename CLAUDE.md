# CLAUDE.md — Instructions for Claude Code

## IMPORTANT: If this is the first message in a session or context was recently compacted, read ~/.claude/plausiden-master-standards.md before doing anything else.

## Project: plausiden-mcp
Model Context Protocol server for the PlausiDen ecosystem. Gives Claude Code and other AI agents structured access to engine generation, injection targets, swarm status, and shield orchestration through a capability-gated, audit-logged MCP interface.

## Part of the PlausiDen Ecosystem
This repo is part of PlausiDen (PLAUSIbly DENiable) protection suite. It is the INTEGRATION LAYER — it connects AI agents to the engine, inject, and swarm crates. It does NOT implement data generation itself; it delegates to those crates (currently stubbed).

## DISAMBIGUATION: This is NOT an Infrastructure Repo
plausiden-mcp exposes the data pollution engine to AI agents. It is NOT related to:
- plausiden-vault (infrastructure credential management for servers)
- plausiden-shield (infrastructure ops monitoring dashboard)
- plausiden-auth (web service authentication library)

If a task mentions "vault," "shield," or "auth" without context, ask which one (infrastructure repo vs PlausiDenOS protection domain vs this MCP integration).

## Architecture
- `src/auth.rs` — Capability-based access control: 6 groups, all off by default, Inject/Swarm require typed acknowledgment
- `src/audit.rs` — Tamper-evident audit log with blake3 hash chain
- `src/config.rs` — Resource limits, server configuration
- `src/server.rs` — MCP JSON-RPC 2.0 server (initialize, tools/list, tools/call, resources, prompts)
- `src/capabilities/` — Generate (7), Inject (3), Swarm (3), Profile (3), Query (3), System (3) = 22 tools
- `src/resources/` — 4 plausiden:// resources (presets, categories, stats, capabilities)
- `src/prompts/` — 3 workflow templates (quick_start, journalist_protection, maximum_protection)
- `src/transport/stdio.rs` — Stdio transport for Claude Code/Cursor
- `src/main.rs` — CLI with clap (--transport, --data-dir)

## Security Model
Everything blocked by default. Six capability groups must be explicitly enabled:
1. **Generate** — memory-only data generation (safe)
2. **Query** — read-only status/stats/audit (safe)
3. **ProfileManagement** — writes config file only
4. **Schedule** — background generation process
5. **Inject** — MODIFIES REAL DATA (requires acknowledgment)
6. **Swarm** — OPENS NETWORK (requires acknowledgment)

## Code Standards
- Rust edition 2021. thiserror for library errors, anyhow for application errors.
- Never unwrap() in library code. All MCP responses must be valid JSON-RPC 2.0.
- No telemetry, no analytics, no phone-home. Zero network unless Swarm capability enabled.
- Every action audit-logged with blake3 hash chain.
- Tool stubs return descriptive messages until engine/inject/swarm crates are integrated.

## Narrative Framing
Civil rights tool restoring the presumption of innocence. NEVER include personal political beliefs or ideology.
