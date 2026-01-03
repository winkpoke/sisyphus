# Product Requirement Document: Sisyphus (General Purpose Agent System)

## 1. Executive Summary
Sisyphus aims to be a robust, open-source **general-purpose AI agent framework**. While maintaining strong coding capabilities, the system is designed to handle a broad spectrum of tasks including system automation, data analysis, research, and content generation. The goal is to provide a provider-agnostic, extensible, and high-performance agent runtime that can power various clients (CLI, IDE extensions, Web) while maintaining a separation between the core logic and the user interface. This project focuses on porting the backend/core capabilities of OpenCode; the OpenCode TUI is not ported.

## 2. Product Scope
The scope includes the migration and generalization of the core business logic, agent orchestration, tool execution, and state management systems from the `opencode` package.

### In Scope
- **Core Agent Logic**: Implementation of versatile agents capable of planning and executing tasks across different domains.
- **Session Management**: Handling conversation history, context compaction, and persistence for long-running tasks.
- **LLM Provider Integration**: Support for multiple providers (OpenAI, Anthropic, Google, Local) via a unified interface.
- **Universal Tooling System**:
    - Built-in tools for system interaction (Bash, File System).
    - **Model Context Protocol (MCP)** implementation to extend capabilities to any domain (Web Search, Database, APIs).
- **Context Awareness**:
    - File and resource tracking.
    - Project/Workspace structure understanding.
- **Server Architecture**: The client/server model allowing remote execution and multiple client connections.
- **CLI Entry Point**: A headless CLI for running the server or executing specific commands.

### Out of Scope
- **OpenCode Terminal UI**: The rich interactive TUI components found in `opencode`.
- **Desktop App Wrapper**: The Electron/Tauri wrappers.
- **Web Interface**: The specific React-based web UI.

## 3. Functional Requirements

### 3.1 Agent System
- **Multi-Agent Support**:
  - **Executor Agent** (Evolution of "Builder"): Full access agent capable of executing commands, editing files, and interacting with external tools to complete tasks.
  - **Planner Agent**: Read-only agent for analysis, strategy formulation, and research without making state-changing modifications.
- **Sub-agents**: Support for specialized sub-agents (e.g., Triage, Researcher, Data Analyst).
- **Prompt Engineering**: Dynamic prompt generation adaptable to the task domain (coding, writing, analysis).
- **Agent Permissions**: Enforce Allow/Ask/Deny gating for tool execution; Ask emits a `PermissionRequest` and pauses the current assistant turn until an explicit approve/deny decision. Tool-call batches resume in order after each decision.

### 3.2 Session & Context Management
- **Conversation History**: Store message history structured as **turns** to maintain logical consistency (e.g., keeping tool calls and results together).
- **Context Compaction**:
  - Token-aware compaction strategies to respect model limits.
  - Support for **Pinned Messages** that persist regardless of context window pressure.
- **Persistence**: Save and load sessions from disk/storage.
- **Context Awareness**:
  - Generic resource tracking (files, URLs, database connections).
  - Environment awareness.

### 3.3 Tooling & Skills
- **Execution Environment**: Safe execution of shell commands.
- **File Operations**: Reading, writing, patching, and diffing files.
- **Knowledge Retrieval**: Semantic or regex-based search.
- **Skills System**:
  - **Declarative Skills**: Define agent capabilities using `SKILL.md` files with YAML frontmatter (name, description) and markdown body for instructions.
  - **Discovery**: Automatically load skills from `.opencode/skill` directories in the workspace.
  - **Dynamic Injection**: Inject skill instructions into the agent's context when relevant to the user's request.
- **MCP Support**: **Crucial for General Purpose**. Full implementation of the Model Context Protocol to connect with external tools (e.g., Google Drive, Slack, Postgres) and data sources.
- **LSP Support**: (Optional/Module) Connect to language servers for coding tasks, treated as just another tool capability.

### 3.4 LLM Abstraction Layer
- **Provider Agnostic**: Unified API for calling different LLMs.
- **Model Configuration**: Support for configuring model parameters (temperature, max tokens).
- **Streaming**: Robust support for streaming responses, ensuring zero data loss from network fragmentation or multi-byte character splits (e.g., using a shared `SSEParser`).

### 3.5 Server & API
- **RPC/HTTP Server**: Expose agent capabilities via an API.
- **Event Bus**: Internal event system for inter-component communication.
- **Events**: Stream `SystemEvent` payloads to clients (including `PermissionRequest`) via the SSE endpoint.
- **Permission Approvals API**: Provide an endpoint to submit approve/deny decisions for Ask-gated tool calls.
- **Security**: Basic authentication and permission management for privileged operations.

### 3.6 Internationalization (i18n)
- **Multi-language Support**: The system shall be designed to support Internationalization (i18n), enabling localization for system messages, logs, and user-facing interactions.

### 3.7 CLI & TUI Experience
- **Interactive TUI**: A rich terminal user interface (TUI) featuring an async event loop that merges user input with backend events.
- **Transcript**: Structured, progressively updating transcript with support for streaming, scrolling, and "stick to bottom" behavior; backend `SystemEvent`s render as concise, end-user-readable entries by default.
- **Command Palette**: A discoverable palette for slash commands triggered by `/`, supporting keyboard navigation and filtering (including `/debug`).
- **Polish**:
  - **Overlays**: Safe rendering of help, errors, and long content (pager) without corrupting the terminal.
  - **Selection**: Ability to select and copy transcript text.
  - **Status**: Structured status bar with session/model/connectivity indicators and a processing spinner.
  - **Layout**: Dynamic transcript header (context name or session ID) and content padding for readability.
- **Permission Prompts**: Decode `PermissionRequest` events into a first-class overlay with Approve/Deny actions to resume the blocked turn; multiple requests are queued FIFO.
- **Debug Mode**: `/debug` toggles raw backend event payload visibility; raw payloads are redacted and truncated when shown.
- **Startup Banner**: Display a branded ASCII banner with version and configuration info on startup.

### 3.8 Slash Command System
- **Interception Layer**: Parses user input starting with `/` before reaching the LLM.
- **Registry**: Supports both built-in Rust functions and custom template-based commands.
- **Command Palette Integration**: Commands shall be discoverable and executable via the TUI's command palette.
- **Extensibility**: Automatically loads custom commands from `.sisyphus/command/*.md`.
- **Templating**: Expands custom command arguments into prompt templates.

## 4. Technical Architecture

### 4.1 Technology Stack
- **Language**: Rust
- **Architecture**: Cargo Workspace (Monorepo)

### 4.2 Key Components (Rust Crates)
- `crates/core`: Core agent logic, session, and memory.
- `crates/common`: Shared utilities, traits, and event bus.
- `crates/tools`: Standard tool implementations (fs, shell, glob, grep).
- `crates/provider`: LLM provider adapters.
- `crates/server`: HTTP/WebSocket API server.
- `crates/cli`: Command-line interface.

### 4.3 Headless Architecture
The system follows a **Headless Agent** design, decoupling the "Brain" (Rust Core) from the "Presentation" (UI).
- **Rust Native World**:
  - `crates/core`: Contains the Agent, Memory, and Logic. Pure Rust, no UI dependencies.
  - `crates/server`: Exposes the core via HTTP/WebSocket and broadcasts `SystemEvents` (JSON).
- **UI Clients**:
  - **VS Code Plugin**: TypeScript extension communicating via WebSocket.
  - **Dioxus TUI/Web**: Rust-based frontends (WASM or Native) connecting to the server.
  - **CLI**: A lightweight client with a feature-gated TUI frontend for interactive sessions, consuming the core directly or via server.

### 4.4 Platform Support
- **Cross-Platform**: The system shall fully support **Linux**, **Windows**, and **macOS** environments for both the server runtime and client tools.

## 5. User Stories
- **General Automation**: As a system admin, I want the agent to check server logs and generate a health report.
- **Research**: As a researcher, I want the agent to search the web, read specific articles, and summarize the findings.
- **Coding**: As a developer, I want the agent to fix linter errors in my codebase.
- **Data Analysis**: As a data analyst, I want the agent to read a CSV file and calculate statistics.
- **Flexibility**: As a user, I want to switch between OpenAI and Claude models depending on the task difficulty.

## 6. Migration Strategy
1.  **Infrastructure Setup**: Initialize the repository and dependency management.
2.  **Core Utilities**: Port logging, event bus, and configuration handling.
3.  **Provider Layer**: Implement the LLM abstraction and at least one provider.
4.  **Session & Agent**: Port the session management and basic agent loop, ensuring prompts are not hardcoded for coding only.
5.  **Tools**: Port the essential file and shell tools, and ensure MCP is a first-class citizen.
6.  **Server**: Implement the API server to expose the functionality.
7.  **CLI Client**: Implement the headless CLI and slash command system.
8.  **Verification**: Unit and integration tests for each module.
