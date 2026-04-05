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
