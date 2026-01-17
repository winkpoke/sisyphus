# OpenSpec Specifications

This directory contains all technical specifications for the Sisyphus project. Each specification defines requirements and scenarios for a specific capability or component.

## Quick Reference

Use this table to quickly find the spec you need:

| Spec | Purpose | Key Requirements |
|------|---------|-----------------|
| [workspace/](workspace/) | Cargo workspace structure | Multi-crate architecture, dependency flow, shared types |
| [common-infra/](common-infra/) | Shared utilities & infrastructure | Event bus, error handling, serialization |
| [agent-core/](agent-core/) | Core agent logic | Agent loop, reasoning, task execution |
| [session-core/](session-core/) | Session & context management | Conversation history, context compaction, persistence |
| [llm-provider/](llm-provider/) | LLM provider abstraction | Provider-agnostic interface, streaming, error handling |
| [tooling/](tooling/) | Tool execution framework | Shell tools, file system, MCP integration |
| [server-core/](server-core/) | HTTP server & API | REST endpoints, SSE streaming, session management |
| [cli-architecture/](cli-architecture/) | CLI design patterns | Command structure, REPL, argument parsing |
| [cli-tui/](cli-tui/) | Terminal UI implementation | MVU pattern, rendering, user interaction |
| [slash-commands/](slash-commands/) | Command system | SlashCommand vs UiCommand, routing, discovery |
| [tui-integration/](tui-integration/) | TUI-server integration | Event streaming, permission prompts, transcript |

---

## Detailed Specifications

### [workspace/](workspace/)

Defines the Cargo workspace structure that enforces separation of concerns across crates (common, core, provider, tools, server, cli). Ensures proper dependency flow with shared types in the common crate preventing circular dependencies.

**Key Topics:**
- Cargo workspace configuration
- Crate organization and responsibilities
- Dependency graph and flow rules

**See:** [workspace/spec.md](workspace/spec.md)

---

### [common-infra/](common-infra/)

Defines shared utilities, traits, and infrastructure used across all crates. Includes the event bus system, error handling conventions, and common serialization patterns.

**Key Topics:**
- Event bus (typed, topic-based)
- Error types and handling
- Shared types and traits
- Serialization conventions

**See:** [common-infra/spec.md](common-infra/spec.md)

---

### [agent-core/](agent-core/)

Defines the core agent logic including the agent loop, reasoning capabilities, and task execution. This is the "brain" of the system that decides what to do based on LLM responses.

**Key Topics:**
- Agent loop (LLM → Tool → Result → Repeat)
- Planning and task breakdown
- Tool selection and execution
- Error recovery and retry logic

**See:** [agent-core/spec.md](agent-core/spec.md)

---

### [session-core/](session-core/)

Defines session management, conversation history, and context handling. Ensures conversations have continuity across multiple turns while respecting token limits.

**Key Topics:**
- Conversation history (structured as turns)
- Context compaction algorithms
- Token-aware history management
- Session persistence
- Context awareness (files, resources)

**See:** [session-core/spec.md](session-core/spec.md)

---

### [llm-provider/](llm-provider/)

Defines the abstraction layer for LLM providers, enabling a unified interface for different AI models (OpenAI, Anthropic, Google, Local).

**Key Topics:**
- Provider trait and interface
- Message format translation
- Streaming responses (SSE handling)
- Error handling and retries
- Provider-specific configuration

**See:** [llm-provider/spec.md](llm-provider/spec.md)

---

### [tooling/](tooling/)

Defines the tool execution framework and built-in tools. Enables agents to interact with the system through shell commands, file operations, and external MCP servers.

**Key Topics:**
- Tool trait and interface
- Shell execution (safe, sandboxed)
- File system operations (read, write, glob, grep)
- Model Context Protocol (MCP) integration
- Skills system (declarative capabilities)

**See:** [tooling/spec.md](tooling/spec.md)

---

### [server-core/](server-core/)

Defines the HTTP server that exposes REST APIs for session management, agent discovery, slash commands, and tool approvals. Provides Server-Sent Events (SSE) endpoint for real-time event streaming.

**Key Topics:**
- HTTP server initialization
- Session management API (CRUD)
- Agent discovery and routing
- Permission approval API
- Real-time event streaming (SSE)
- Slash command discovery
- Unified chat service

**See:** [server-core/spec.md](server-core/spec.md)

---

### [cli-architecture/](cli-architecture/)

Defines the CLI design patterns, command structure, and REPL behavior. Ensures CLI is a thin layer over core functionality.

**Key Topics:**
- Command structure (subcommands, flags)
- REPL implementation
- Argument parsing conventions
- Error display and user feedback

**See:** [cli-architecture/spec.md](cli-architecture/spec.md)

---

### [cli-tui/](cli-tui/)

Defines the optional Terminal User Interface (TUI) implementation using Model-View-Update (MVU) pattern. Provides a rich, interactive chat experience in the terminal.

**Key Topics:**
- MVU architecture (pure update function, centralized actions)
- Rendering and layout
- User interaction (keyboard, mouse)
- Theme and styling
- Permission prompt overlays
- Transcript display and scrolling
- Command palette integration

**See:** [cli-tui/spec.md](cli-tui/spec.md)

---

### [slash-commands/](slash-commands/)

Defines the dual command system separating server-side SlashCommands from client-side UiCommands. Enables extensible command system with proper routing.

**Key Topics:**
- SlashCommand (server-side prompt templates)
- UiCommand (client-side actions)
- Command routing (client handles local, forwards to server)
- Command discovery and metadata
- No command effects in chat responses
- Custom command loading from `.sisyphus/command/`

**See:** [slash-commands/spec.md](slash-commands/spec.md)

---

### [tui-integration/](tui-integration/)

Defines the integration between the TUI and the server, focusing on event streaming, permission handling, and transcript management.

**Key Topics:**
- Backend event streaming (SSE)
- Permission request display and handling
- Transcript rendering from SystemEvents
- Debug mode toggle
- Command palette integration
- Status bar and session indicators

**See:** [tui-integration/spec.md](tui-integration/spec.md)

---

## How to Use This Directory

### For Feature Implementation

1. **Identify the relevant spec**: Use the table above or search by keyword
2. **Read the spec**: Review all requirements and scenarios
3. **Check for design.md**: Many specs have additional technical design documents
4. **Create a change proposal**: See [openspec/AGENTS.md](../AGENTS.md) for the workflow
5. **Implement against the spec**: Ensure all scenarios pass

### For Understanding the System

1. **Start with workspace/**: Understand the overall architecture
2. **Read agent-core/ and session-core/**: Understand core concepts
3. **Explore component specs**: Dive into specific areas of interest
4. **Review PRD.md**: Connect technical specs to product requirements

### For Troubleshooting

1. **Identify the failing component**: Which crate or feature is having issues?
2. **Check the spec**: What are the expected behaviors?
3. **Review scenarios**: Does your implementation match the scenarios?
4. **Check design.md**: Are there implementation patterns you're missing?

---

## Spec Conventions

Each spec directory contains:
- **spec.md**: Requirements and scenarios (authoritative)
- **design.md**: Technical patterns and implementation details (optional)

### Scenario Format

Scenarios follow the Gherkin-like format:
```markdown
#### Scenario: Descriptive name
- **GIVEN** initial state
- **WHEN** action occurs
- **THEN** expected result
```

### Requirement Format

Requirements use normative language (SHALL/MUST):
```markdown
### Requirement: Feature Name
The system SHALL provide the feature.

#### Scenario: Success case
- **WHEN** valid input provided
- **THEN** correct result returned
```

---

## Related Documentation

- **[AGENTS.md](../AGENTS.md)** - Main navigation hub
- **[PRD.md](../../PRD.md)** - Product requirements and architecture
- **[TEST_STRATEGY.md](../../TEST_STRATEGY.md)** - Testing approach
- **[project.md](../project.md)** - Code conventions and patterns
- **[AGENTS.md](../AGENTS.md)** - OpenSpec workflow instructions
