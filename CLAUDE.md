# CLAUDE.md — Instructions for Claude Code

## IMPORTANT: If this is the first message in a session or context was recently compacted, read this entire file before doing anything else.

## Project: plausiden-mcp
Model Context Protocol server for the PlausiDen ecosystem. Gives Claude Code and other AI agents structured access to engine generation, injection targets, swarm status, and shield orchestration.

## Part of the PlausiDen Ecosystem
This repo is part of PlausiDen (PLAUSIbly DENiable) protection suite.

## Architecture
- Rust binary communicating over stdin/stdout using MCP JSON-RPC protocol
- Tool definitions in `src/tools.rs` — each tool maps to a PlausiDen capability
- Protocol handling in `src/protocol.rs` — JSON-RPC parsing and routing
- Engine/inject/swarm integrations will be added as those crates stabilize

## MCP Tools Provided
- `plausiden_generate` — generate synthetic artifacts
- `plausiden_list_generators` — list available generators with status
- `plausiden_configure_profile` — set user profile for generation
- `plausiden_list_targets` — list injection targets on this system
- `plausiden_inject` — inject artifacts into OS data stores
- `plausiden_swarm_status` — P2P swarm node status
- `plausiden_threat_assessment` — detect forensic tools, evaluate posture
- `plausiden_ecosystem_status` — all repo build/test status

## Code Standards
- Rust edition 2024. thiserror for errors. Never unwrap() in library code.
- All MCP responses must be valid JSON-RPC.
- No telemetry, no analytics, no phone-home.
- Tool stubs return descriptive messages until integrations are ready.

## Narrative Framing
Civil rights tool. NEVER include personal political beliefs or ideology.
