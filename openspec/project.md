# Project Context

## Purpose

Sisyphus is a robust, open-source general-purpose AI agent framework ported from the core logic of OpenCode. It provides a flexible, headless, and extensible system for building AI agents that can interact with the world through tools and APIs. The framework focuses on high-performance agent runtime that can power various clients (CLI, IDE extensions, Web) while maintaining strict separation between core logic and user interface.

## Tech Stack

- **Language**: Rust 1.75+
- **Architecture**: Cargo Workspace (Monorepo)
- **Runtime**: Tokio (async runtime)
- **Serialization**: `serde` for JSON/bincode serialization
- **Templating**: `minijinja` for dynamic system prompts
- **Testing**: `mockall` (mocking), `tokio-test` (async), `tempfile` (fs), `wiremock` (HTTP), `insta` (snapshots)
- **Server**: `axum` (HTTP), `tokio` (WebSockets), SSE for streaming
- **CLI**: `clap` (argument parsing), `ratatui` (TUI, optional)

## Project Conventions

### Code Style

**Formatting:**
- Use `cargo fmt` for consistent formatting
- Follow Rust standard naming conventions (snake_case for functions/variables, PascalCase for types)
- Maximum line length: 100 characters

**Naming Conventions:**
- **Modules**: `snake_case` (e.g., `agent_core`, `session_manager`)
- **Types**: `PascalCase` (e.g., `Agent`, `Session`, `Message`)
- **Functions/Methods**: `snake_case` (e.g., `chat_request`, `execute_tool`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_TOKEN_LIMIT`)
- **Async functions**: Same naming, but must return `Future` type

**Error Handling:**
- Use `thiserror` for custom error types
- Prefer `Result<T, E>` over panic/unwrap in production code
- Provide context for errors using `anyhow`'s `.context()`
- Never suppress errors with empty catch blocks

**Documentation:**
- Public APIs must have `///` doc comments
- Use `cargo doc` to verify documentation builds
- Include examples for complex APIs
- Reference relevant specifications in doc comments (e.g., "See `server-core/spec.md`")

### Architecture Patterns

**Cargo Workspace Structure:**
```
crates/
├── common/      # Shared types, traits, event bus - NO business logic
├── core/        # Agent logic, session management, memory
├── provider/    # LLM provider adapters (OpenAI, Anthropic, etc.)
├── tools/       # Standard tool implementations (fs, shell)
├── server/      # HTTP/WebSocket API server
├── cli/         # CLI entry point
├── cli-core/    # Shared CLI library (commands, REPL, completer)
└── tui/         # Optional TUI frontend (requires `tui` feature)
```

**Dependency Flow (Enforce to prevent circular dependencies):**
- `common` crate has NO dependencies on other crates in the workspace
- All other crates MAY depend on `common`
- NO crate may depend on `cli` or `cli-core` (entry points only)
- `core` depends on: `common`, `provider`, `tools`
- `server` depends on: `common`, `core`
- `cli` depends on: `common`, `core`, `cli-core` (optionally `tui`)

**Headless Architecture:**
- **Core Logic** (`crates/core`): Pure agent behavior, no UI concerns
- **Transport Layer** (`crates/server`): Thin HTTP/WebSocket adapter
- **UI Clients**: VS Code Plugin, Dioxus TUI/Web, CLI (all optional)

**Trait-Based Design:**
- `LLMProvider` trait: Unified interface for all LLM providers
- `Tool` trait: Common interface for system tools
- `EventBus` trait: Subscribe/publish pattern for events

**Event-Driven:**
- Use `EventBus` for loose coupling between components
- Events are typed structs implementing `Event` trait
- Subscribe to specific topics for efficiency or `subscribe_raw()` for global auditing

### Testing Strategy

**Testing Pyramid (70/20/10):**
- **Unit Tests (70%)**: Focus on `core` logic using mocks (`mockall`) for LLM and Tools. Fast, deterministic, run on every commit.
- **Integration Tests (20%)**: Verify `providers` format requests correctly and `tools` interact with OS in sandboxed environments.
- **E2E Tests (10%)**: Black box testing of CLI and Server using replay mechanism for LLM responses.

**Key Principles:**
- **No live API calls**: Use `wiremock` for HTTP providers, `mockall` for LLM/Tool traits
- **Sandboxing**: Use `tempfile::tempdir()` for all filesystem tool tests
- **Snapshot testing**: Use `insta` for verifying generated prompts and JSON schemas
- **Deterministic**: All unit tests must be deterministic (no time/sleep dependencies)

**Required Tooling (dev-dependencies):**
- `mockall` - Mock `LLMProvider` and `Tool` traits
- `tokio-test` - Test async functions deterministically
- `tempfile` - Safely test filesystem tools
- `wiremock` - Mock HTTP endpoints for provider tests
- `insta` - Snapshot testing for prompts/JSON

See [TEST_STRATEGY.md](TEST_STRATEGY.md) for comprehensive testing guidelines.

### Git Workflow

**Branching Strategy:**
- `main` - Stable, production-ready code
- `develop` - Integration branch for features
- `feature/<name>` - Feature development
- `fix/<name>` - Bug fixes
- `refactor/<name>` - Code refactoring

**Commit Conventions:**
- Use conventional commits: `type(scope): description`
  - `feat(core): add context compaction algorithm`
  - `fix(server): resolve permission request deadlock`
  - `docs: update AGENTS.md with new navigation`
  - `test(core): add unit tests for agent loop`
  - `refactor(provider): extract common request formatting`
- Commit message body should reference specs when relevant: "Implements `agent-core/spec.md` Requirement: Agent Loop"

**Pull Request Requirements:**
- All CI checks must pass
- At least one reviewer approval required
- Reference related issues or specs in PR description
- Include tests for new functionality
- Update documentation (specs, AGENTS.md) as needed

**OpenSpec Integration:**
- Create spec proposals in `openspec/changes/<id>/` BEFORE implementing features
- Implement against spec deltas, validate with `openspec validate <id> --strict`
- Archive completed changes: `openspec archive <id> --yes`
- See [openspec/AGENTS.md](openspec/AGENTS.md) for complete workflow

## Domain Context

**Agent System:**
- **Planner Agent**: Read-only agent for analysis, strategy, research without state changes
- **Executor Agent**: Full access agent capable of executing commands, editing files, using tools
- **Sub-agents**: Specialized agents (Triage, Researcher, Data Analyst) for specific tasks

**Tooling:**
- **Built-in Tools**: Shell execution, file system operations (read/write), grep search
- **MCP Support**: Model Context Protocol for connecting to external tools (Google Drive, Slack, Postgres)
- **Skills System**: Declarative skills defined in `.sisyphus/skill/*.md` files with YAML frontmatter

**Session Management:**
- **Conversation History**: Structured as turns (maintain tool calls + results together)
- **Context Compaction**: Token-aware strategies with pinned messages that persist
- **Persistence**: Save/load sessions from disk/storage
- **Context Awareness**: Track files, URLs, database connections, environment

**Multi-Agent Routing:**
- Session assignment to specific agents via `POST/PUT /api/v1/sessions`
- Agent discovery via `/api/v1/agents`
- Chat requests routed to session's active agent
- Default agent is `plan` when none specified

**Permission System:**
- **Allow/Ask/Deny** gating for tool execution
- Ask emits `PermissionRequest` and blocks current turn
- Tool-call batches processed in order; each Ask blocks until decision
- Deny appends deterministic result: `Permission denied: user rejected tool execution.`

## Important Constraints

**Headless Design:**
- Core logic MUST NOT depend on UI libraries
- UI clients are optional and replaceable
- All state managed in core; UI is just a view/controller

**Provider Agnostic:**
- System MUST support multiple LLM providers (OpenAI, Anthropic, Google, Local)
- Provider-specific logic isolated in `crates/provider/`
- Core agent works with generic `LLMProvider` trait

**Performance Requirements:**
- Streaming responses MUST use shared `SSEParser` for zero data loss
- Support split network chunks and multi-byte characters
- Efficient event distribution via topic-based subscriptions

**Security:**
- Tool execution MUST be safe and sandboxed where possible
- Permission system REQUIRED for dangerous operations
- No arbitrary code execution from untrusted sources

**Error Handling:**
- NEVER use `unwrap()`, `expect()`, or `panic!()` in production code
- Provide context for errors to aid debugging
- Graceful degradation on failures

**Backward Compatibility:**
- Changes to public APIs require spec proposals
- Breaking changes MUST be marked `**BREAKING**` in proposals
- Provide migration paths for deprecated features

## External Dependencies

**LLM Providers:**
- OpenAI API (https://platform.openai.com/docs)
- Anthropic Claude API (https://docs.anthropic.com)
- Google Gemini API (https://ai.google.dev/docs)
- Local LLMs (Ollama, llama.cpp)

**Model Context Protocol (MCP):**
- MCP specification for tool integration
- MCP-compatible servers for external capabilities

**Rust Ecosystem:**
- Tokio (async runtime): https://tokio.rs
- Axum (HTTP server): https://docs.rs/axum
- Serde (serialization): https://serde.rs
- Thiserror (error handling): https://docs.rs/thiserror
- Anyhow (error context): https://docs.rs/anyhow

**Documentation Standards:**
- Follow Rust API guidelines: https://rust-lang.github.io/api-guidelines/
- Async Rust book: https://rust-lang.github.io/async-book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
