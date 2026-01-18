# Sisyphus - Agent Reference Guide

## 📑 Quick Navigation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[PRD.md](PRD.md)** | Product requirements, scope, architecture | Understanding what we're building |
| **[openspec/specs/](openspec/specs/)** | Technical specifications & scenarios | Implementing features |
| **[openspec/AGENTS.md](openspec/AGENTS.md)** | OpenSpec change workflow | Creating/applying change proposals |
| **[TEST_STRATEGY.md](TEST_STRATEGY.md)** | Testing strategy & guidelines | Writing tests |
| **[openspec/project.md](openspec/project.md)** | Code conventions & patterns | Following project standards |

## ⚡ Critical Runtime Behaviors

### Permission System
- **Ask-gated tool execution**: Emits `PermissionRequest` and blocks current turn
- Tool calls processed in order; later calls held until blocking call resolves
- Clients render permission prompts (operation/tool_name/call_id), queue multiple requests FIFO

### Command System
- **SlashCommand**: Server-side prompt templates (expanded by agent, e.g., custom prompts)
- **UiCommand**: Client-side UI actions (handled locally: `/clear`, `/debug`, `/think`)
- **Routing**: Clients route UiCommands locally, forward SlashCommands to server
- **No Command Effects**: Chat responses MUST NOT carry UI/session lifecycle effects

### Multi-Agent System
- **Agent Discovery**: `/api/v1/agents` lists available agents
- **Session Assignment**: Sessions can be created/updated with specific agents via `POST/PUT /api/v1/sessions`
- **Default Agent**: `plan` is the default when no agent specified

### Streaming & Event Bus
- **SSE Events**: Stream `EventEnvelope<SystemEvent>` to clients with envelope `id` set as SSE `id` field
- **Event Bus**: Components publish/subscribe `EventEnvelope<SystemEvent>` with stable metadata (`id`, `timestamp_ms`)
- **LLM Streaming**: MUST use shared `SSEParser` (`crates/provider/src/sse.rs`) for correct handling of split network chunks and multi-byte characters
- **Subscription Patterns**: Use `subscribe_raw()` for global auditing or specific topics for efficiency

## 🛠️ Development Quick Reference

### Running the Project
```bash
cargo build --release           # Build CLI-only (default)
cargo build --release --features tui  # Build with TUI support
cargo test                       # Run all tests
```

### Debug Mode
```bash
cargo run --release --features dev_debug
# HTTP request/response logging to `openai_debug.log` from `crates/provider/src/openai.rs`
```

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

## 📖 Related Documentation

- **[README.md](README.md)** - Project overview and quick start
- **[openspec/specs/](openspec/specs/)** - Detailed technical specifications with scenarios
- **[TEST_STRATEGY.md](TEST_STRATEGY.md)** - Comprehensive testing strategy (unit, integration, E2E)
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines and testing workflow

## 🧪 Testing Quick Reference

### Running Tests & Coverage
```bash
# Run all tests
cargo test

# Generate coverage reports
./scripts/coverage.sh

# View HTML report
open target/tarpaulin/index.html
```

### Test Organization
- **Unit tests**: `#[cfg(test)]` modules within source files
- **Integration tests**: `tests/` directories at crate level
- **Test fixtures**: `tests/fixtures/` for reusable inputs
- **Golden files**: `tests/golden/` for expected outputs
- **Snapshots**: Managed by `insta` (review with `cargo insta review`)

### Coverage Targets
- **Critical (90%+)**: `tools/src/fs.rs`, `tools/src/cmd.rs`, `core/src/agent.rs`, `core/src/session.rs`
- **High (80%+)**: `provider/src/openai.rs`, `server/src/lib.rs`, `core/src/session/context.rs`
- **Medium (70%+)**: `cli-core/`, `common/src/`, `client/`
- **Low (60%+)**: `tui/` (logic only)

> See [TEST_STRATEGY.md](TEST_STRATEGY.md) for detailed testing patterns and [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow.

---
