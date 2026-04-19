# Architecture — PlausiDen MCP Server

## System Diagram

```
┌─────────────────────────────────────────────────────┐
│                   MCP Client                        │
│  (Claude Code / Claude Desktop / Cursor / Custom)   │
└──────────────────┬──────────────────────────────────┘
                   │ JSON-RPC 2.0 (stdin/stdout)
                   │
┌──────────────────▼──────────────────────────────────┐
│                 plausiden-mcp                        │
│                                                     │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ Transport   │  │ Capability   │  │ Audit Log │  │
│  │ (stdio/http)│→ │ Gate (auth)  │→ │ (blake3)  │  │
│  └─────────────┘  └──────┬───────┘  └───────────┘  │
│                          │                          │
│  ┌───────────────────────▼───────────────────────┐  │
│  │              Tool Dispatch                    │  │
│  │                                               │  │
│  │  ┌──────────┐ ┌────────┐ ┌───────┐ ┌──────┐  │  │
│  │  │ Generate │ │ Inject │ │ Swarm │ │Query │  │  │
│  │  │ (7 tools)│ │(3 tool)│ │(3 tool│ │(3)   │  │  │
│  │  └──────────┘ └────────┘ └───────┘ └──────┘  │  │
│  │  ┌──────────┐ ┌────────┐                      │  │
│  │  │ Profile  │ │ System │                      │  │
│  │  │ (3 tools)│ │(3 tool)│                      │  │
│  │  └──────────┘ └────────┘                      │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌───────────────┐  ┌──────────┐  ┌────────────┐   │
│  │ Resources (4) │  │Prompts(3)│  │ Config     │   │
│  └───────────────┘  └──────────┘  └────────────┘   │
└─────────────────────────────────────────────────────┘
         │ (future)          │ (future)
         ▼                   ▼
   plausiden-engine    plausiden-swarm
   plausiden-inject
```

## Data Flow

1. MCP client sends JSON-RPC request on stdin
2. Transport layer parses the request
3. Server routes to the appropriate handler (initialize, tools/call, resources/read, prompts/get)
4. For tool calls: capability gate checks if the required capability is enabled
5. If denied: audit log records denial, error returned
6. If allowed: audit log records call (with param hash, not params), tool handler executes
7. Response serialized as JSON-RPC and written to stdout

## Threat Model

**In scope:**
- Preventing accidental execution of dangerous tools (inject, swarm) without explicit consent
- Tamper-evident audit trail for all actions (hash chain makes retroactive log editing detectable)
- Resource limits preventing runaway generation or injection
- Zero network policy unless swarm is explicitly enabled

**Out of scope:**
- Protecting against a malicious MCP client (the client is trusted as the user's agent)
- Encrypting the audit log at rest (future: encrypt with age)
- Protecting against a compromised host OS

## Key Design Decisions

1. **Capability-based access control** — not role-based. Each tool maps to exactly one capability. No capability grants access to tools in another group.
2. **Audit-first** — every action is logged before execution. The hash chain means you can prove what happened and when.
3. **Stubs over missing features** — tools return descriptive stubs until engine/inject/swarm crates are integrated. The MCP interface is stable; the implementation will be swapped in.
4. **Stdio-first transport** — HTTP is optional and localhost-only. Most MCP clients use stdio.

## Future Directions

- Integration with plausiden-engine for real data generation
- Integration with plausiden-inject for OS data store writes
- Integration with plausiden-swarm for P2P network participation
- HTTP+SSE transport for web-based MCP clients
- Embedded mode for SacredVote-Desktop (Tauri app embeds MCP server)

---

## Out of Scope

Per v1.2 §G.3. MCP is the **AI-agent-to-engine integration layer**.
It exposes capability-gated tools through JSON-RPC 2.0. It does NOT:

- **Implement data generation itself.** All Generate tools delegate
  to `plausiden-engine` (currently stubbed); MCP never synthesizes
  browsing history, contacts, or network traffic in this crate.
- **Write to OS data stores.** Inject tools are stubs pending
  `plausiden-inject` integration. When live, the writes happen in
  plausiden-inject — MCP is the dispatcher, not the driver.
- **Run a P2P network.** Swarm tools delegate to
  `plausiden-swarm`. MCP's role is capability-gating + audit, not
  packet relay.
- **Protect against a malicious MCP client.** The client is
  trusted as the user's agent. An attacker with control of Claude
  Code, Cursor, or a custom client can invoke any enabled
  capability. Threat model covers the server, not the client.
- **Encrypt the audit log at rest.** The blake3 hash chain is
  tamper-EVIDENT, not tamper-proof. An adversary with read + write
  access to `~/.local/share/plausiden-mcp/audit.log` can truncate
  or replay but cannot silently modify history without breaking
  the chain. Future: age-encrypt the log.
- **Protect against a compromised host OS.** If the kernel or
  user account is compromised, MCP's capability gating is moot.
  See `plausiden-desktop` / `plausiden-os-for-mobile` for the
  Tier-2/3 pathways that handle that threat model.
- **Provide a UI.** MCP is an integration LAYER. The UI is
  whatever MCP client the user runs (Claude Code, Cursor, a
  future Tauri-embedded client).
- **Act as a general-purpose RPC framework.** MCP implements a
  specific JSON-RPC 2.0 protocol per the Model Context Protocol
  spec. Custom protocols belong in separate crates.

Scope creep in any of these directions would blur the capability
model that MCP exists to enforce.
