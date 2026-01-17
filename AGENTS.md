# Sisyphus - Agent Reference Guide

## 📑 Quick Navigation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[PRD.md](PRD.md)** | Product requirements, scope, architecture | Understanding what we're building |
| **[openspec/specs/](openspec/specs/)** | Technical specifications & scenarios | Implementing features |
| **[openspec/AGENTS.md](openspec/AGENTS.md)** | OpenSpec change workflow | Creating/applying change proposals |
| **[TEST_STRATEGY.md](TEST_STRATEGY.md)** | Testing strategy & guidelines | Writing tests |
| **[openspec/project.md](openspec/project.md)** | Code conventions & patterns | Following project standards |

---

## 📚 Document Hierarchy

```
Sisyphus Project
├── PRD.md                      # WHY: Product vision & requirements
├── AGENTS.md (this file)       # HOW: Central navigation hub
├── TEST_STRATEGY.md            # VERIFY: Testing approach
└── openspec/
    ├── AGENTS.md              # WORKFLOW: Change proposal process
    ├── project.md             # CONVENTIONS: Code style & patterns
    └── specs/                 # TRUTH: Detailed technical specs
        ├── agent-core/spec.md
        ├── server-core/spec.md
        ├── session-core/spec.md
        ├── llm-provider/spec.md
        ├── cli-tui/spec.md
        ├── tooling/spec.md
        ├── common-infra/spec.md
        ├── workspace/spec.md
        └── ... (more specs)
```

---

## 🚀 When to Use Each Document

### Quick Decision Tree

**"What are we building?"** → Read [PRD.md](PRD.md)
- Executive summary, scope, functional requirements
- Architecture overview, user stories

**"How do I implement this feature?"** → Follow this path:
1. Check [openspec/specs/](openspec/specs/) for the relevant capability spec
2. Read [openspec/AGENTS.md](openspec/AGENTS.md) for the change proposal workflow
3. Reference [openspec/project.md](openspec/project.md) for code conventions
4. Use [TEST_STRATEGY.md](TEST_STRATEGY.md) for testing guidelines

**"I need to create a spec change proposal"** → Read [openspec/AGENTS.md](openspec/AGENTS.md)
- Complete OpenSpec workflow (Create → Implement → Archive)
- Proposal structure, spec delta format

**"What's the testing strategy?"** → Read [TEST_STRATEGY.md](TEST_STRATEGY.md)
- Testing pyramid (unit, integration, E2E)
- Recommended tooling, component-specific strategies

---

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

---

## ⚡ Critical Runtime Behaviors

These are the most important behaviors that agents must know. For detailed specs, see [openspec/specs/](openspec/specs/).

### Permission System
- **Ask-gated tool execution**: Emits `PermissionRequest` and blocks current turn
- Tool calls processed in order; later calls held until blocking call resolves
- Deny appends deterministic result: `Permission denied: user rejected tool execution.`
- Clients render permission prompts (operation/tool_name/call_id), queue multiple requests FIFO, and submit approve/deny decisions

### Command System
- **SlashCommand**: Server-side prompt templates (expanded by agent, e.g., custom prompts)
- **UiCommand**: Client-side UI actions (handled locally: `/clear`, `/debug`)
- **Routing**: Clients route UiCommands locally, forward SlashCommands to server
- **No Command Effects**: Chat responses MUST NOT carry UI/session lifecycle effects; use UiCommands + explicit APIs
- **Discovery**: Server exposes SlashCommand metadata; clients merge with local UiCommands for unified Command Palette

### Multi-Agent System
- **Agent Discovery**: `/api/v1/agents` lists available agents
- **Session Assignment**: Sessions can be created/updated with specific agents via `POST/PUT /api/v1/sessions`
- **Default Agent**: `plan` is the default when no agent specified
- **Routing**: Chat requests routed to the session's active agent

### Streaming
- **SSE Events**: Stream `SystemEvent` payloads to clients (including `PermissionRequest`) via the SSE endpoint
- **LLM Streaming**: MUST use shared `SSEParser` (`crates/provider/src/sse.rs`) for correct handling of split network chunks and multi-byte characters

### Event Bus
- **Typed, Topic-Based**: Components must use `subscribe_raw()` for global auditing or specific topics for efficiency

### Unified Chat Service
- All chat requests and command effects are handled by a single core application service to ensure consistent state transitions across all transports

### System Prompts
- Supports `minijinja` templating for dynamic system prompt generation, configurable via `system_prompt_template`

### Internationalization
- System designed to support Internationalization (i18n) for system messages, logs, and user-facing interactions

---

## 🛠️ Development Quick Reference

### Running the Project
```bash
# Build CLI-only (default)
cargo build --release

# Build with TUI support
cargo build --release --features tui

# Run tests
cargo test
```

### Debug Mode
```bash
# Enable debug instrumentation
cargo run --release --features dev_debug
```

**Current debug instrumentation:**
- HTTP request/response logging to `openai_debug.log` (headers, payloads, timestamps) from `crates/provider/src/openai.rs`

**Adding debug code:**
- Wrap debug code with `#[cfg(feature = "dev_debug")]`
- Define `dev_debug = []` in crate's `Cargo.toml` to participate in workspace feature

### Key File Locations
- **Agent logic**: `crates/core/`
- **HTTP server**: `crates/server/`
- **LLM providers**: `crates/provider/`
- **Tools**: `crates/tools/`
- **Common utilities**: `crates/common/`
- **CLI entry**: `crates/cli/`
- **TUI frontend**: `crates/tui/` (optional, requires `tui` feature)

### Architecture Overview

**Cargo Workspace Structure:**
```
sisyphus/
├── crates/
│   ├── common/      # Shared utilities, traits, event bus
│   ├── core/        # Agent logic, session management, memory
│   ├── provider/    # LLM provider adapters
│   ├── tools/       # Standard tools (fs, shell)
│   ├── server/      # HTTP/WebSocket API server
│   ├── cli/         # CLI entry point
│   ├── cli-core/    # Shared CLI library (commands, REPL, completer)
│   └── tui/         # Optional TUI frontend (requires `--features tui`)
└── Cargo.toml       # Workspace configuration
```

**Headless Architecture:**
- **Rust Native World**: `crates/core` (Agent, Memory, Logic, ChatService) + `crates/server` (transport adapter)
- **UI Clients**: VS Code Plugin, Dioxus TUI/Web, CLI (headless or with optional TUI)

---

## 📖 Related Documentation

- **[README.md](README.md)** - Project overview and quick start
- **[openspec/specs/](openspec/specs/)** - Detailed technical specifications with scenarios
- **[TEST_STRATEGY.md](TEST_STRATEGY.md)** - Comprehensive testing strategy (unit, integration, E2E)
- **[PRD.md](PRD.md)** - Product requirements and architecture (scope, functional requirements, user stories)
- **[openspec/AGENTS.md](openspec/AGENTS.md)** - OpenSpec workflow instructions (Create → Implement → Archive)
- **[openspec/project.md](openspec/project.md)** - Code conventions and architectural patterns
