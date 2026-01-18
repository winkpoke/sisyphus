# Sisyphus - Agent Reference Guide

## 📑 Quick Navigation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[PRD.md](PRD.md)** | Product requirements, scope, architecture | Understanding what we're building |
| **[openspec/specs/](openspec/specs/)** | Technical specifications & scenarios | Implementing features |
| **[openspec/AGENTS.md](openspec/AGENTS.md)** | OpenSpec change workflow | Creating/applying change proposals |
| **[TEST_STRATEGY.md](TEST_STRATEGY.md)** | Testing strategy & guidelines | Writing tests |
| **[openspec/project.md](openspec/project.md)** | Code conventions & patterns | Following project standards |

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
```

## ⚡ Critical Runtime Behaviors

These are the most important behaviors that agents must know. For detailed specs, see [openspec/specs/](openspec/specs/).

### Permission System
- **Ask-gated tool execution**: Emits `PermissionRequest` and blocks current turn
- Tool calls processed in order; later calls held until blocking call resolves
- Clients render permission prompts (operation/tool_name/call_id), queue multiple requests FIFO

### Command System
- **SlashCommand**: Server-side prompt templates (expanded by agent, e.g., custom prompts)
- **UiCommand**: Client-side UI actions (handled locally: `/clear`, `/debug`)
- **Routing**: Clients route UiCommands locally, forward SlashCommands to server
- **No Command Effects**: Chat responses MUST NOT carry UI/session lifecycle effects

### Multi-Agent System
- **Agent Discovery**: `/api/v1/agents` lists available agents
- **Session Assignment**: Sessions can be created/updated with specific agents via `POST/PUT /api/v1/sessions`
- **Default Agent**: `plan` is the default when no agent specified

### Streaming & Event Bus
- **SSE Events**: Stream `SystemEvent` payloads to clients (including `PermissionRequest`) via the SSE endpoint
- **LLM Streaming**: MUST use shared `SSEParser` (`crates/provider/src/sse.rs`) for correct handling of split network chunks and multi-byte characters
- **Event Bus**: Components must use `subscribe_raw()` for global auditing or specific topics for efficiency

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

## 🧪 Test Data Management

### Directory Structure

Tests use organized directories for maintainability:

```
crates/
├── core/
│   └── tests/
│       ├── fixtures/         # Reusable test input data (JSON, YAML)
│       ├── golden/          # Expected output files for comparison tests
│       └── snapshots/       # Insta snapshots (managed by cargo insta)
├── tools/
│   └── tests/
│       ├── fixtures/
│       └── golden/
├── provider/
│   └── tests/
│       ├── fixtures/
│       └── golden/
└── ...
```

### Guidelines

- **Fixtures**: Reusable test inputs (request/response examples, configs)
- **Golden files**: Expected outputs for file comparison tests
- **Snapshots**: Managed automatically by `insta` (review with `cargo insta review`)
- Keep fixtures small and focused on specific test cases
- Document fixture purpose in file headers or README
- Update golden files alongside code changes (version together)

### Running Coverage

```bash
# Generate coverage reports
./scripts/coverage.sh

# View HTML report
open target/tarpaulin/index.html
```

CI/CD automatically runs tests and uploads coverage to Codecov on every push and PR.

- **[PRD.md](PRD.md)** - Product requirements and architecture (scope, functional requirements, user stories)
- **[openspec/AGENTS.md](openspec/AGENTS.md)** - OpenSpec workflow instructions (Create → Implement → Archive)
- **[openspec/project.md](openspec/project.md)** - Code conventions and architectural patterns

## Testing Patterns

### Unit Tests
- Test single functions/modules in `#[cfg(test)]` modules
- Use `tokio::test` for async tests
- Use mocks (`mockall`, `wiremock`) to isolate dependencies
- Keep tests focused and fast (<1s each)
- Use descriptive test names: `test_<feature>_<scenario>`

### Integration Tests
- Test end-to-end flows in `tests/` directories
- Use real dependencies for realistic scenarios
- Test API contracts with `wiremock`
- Verify request/response formats match specifications

### Test Organization
```
crates/
├── core/
│   └── tests/
│       ├── fixtures/     # Reusable test inputs
│       ├── golden/      # Expected outputs
│       └── *_test.rs    # Integration tests
├── provider/
│   └── src/*.rs
│       └── #[cfg(test)] mod tests  # Unit tests
└── tools/
    └── src/*.rs
        └── #[cfg(test)] mod tests  # Unit tests
```

### Mocking Guidelines
- Use `wiremock` for HTTP endpoints (provider tests)
- Use `mockall` for trait implementations (provider, tool traits)
- Keep mock behavior close to real implementation
- Test both success and error paths
- Verify mock responses match API contracts

### Snapshot Testing
Use `insta` for testing generated content:
```bash
# Generate snapshots
cargo test

# Review snapshot changes
cargo insta review
```

Snapshot targets:
- System prompts (agent/prompt.rs)
- Tool schemas (provider implementations)
- JSON request/response formats
- Error messages
