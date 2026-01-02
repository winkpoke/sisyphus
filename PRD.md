# Product Requirement Document: Sisyphus (General Purpose Agent System)

## 1. Executive Summary
Sisyphus aims to be a robust, open-source **general-purpose AI agent framework**, ported from the core logic of [OpenCode](https://opencode.ai). While maintaining strong coding capabilities, the system is designed to handle a broad spectrum of tasks including system automation, data analysis, research, and content generation. The goal is to provide a provider-agnostic, extensible, and high-performance agent runtime that can power various clients (CLI, IDE extensions, Web) while maintaining a separation between the core logic and the user interface. This project specifically focuses on porting the backend/core capabilities of OpenCode, excluding the Terminal User Interface (TUI).

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
- **Terminal User Interface (TUI)**: The rich interactive TUI components found in `opencode`.
- **Desktop App Wrapper**: The Electron/Tauri wrappers.
- **Web Interface**: The specific React-based web UI.

## 3. Functional Requirements

### 3.1 Agent System
- **Multi-Agent Support**:
  - **Executor Agent** (Evolution of "Builder"): Full access agent capable of executing commands, editing files, and interacting with external tools to complete tasks.
  - **Planner Agent**: Read-only agent for analysis, strategy formulation, and research without making state-changing modifications.
- **Sub-agents**: Support for specialized sub-agents (e.g., Triage, Researcher, Data Analyst).
- **Prompt Engineering**: Dynamic prompt generation adaptable to the task domain (coding, writing, analysis).

### 3.2 Session & Context Management
- **Conversation History**: Store and retrieve message history.
- **Context Compaction**: Algorithms to summarize or truncate history to fit context windows.
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
- **Streaming**: Support for streaming responses from LLMs.

### 3.5 Server & API
- **RPC/HTTP Server**: Expose agent capabilities via an API.
- **Event Bus**: Internal event system for inter-component communication.
- **Security**: Basic authentication and permission management for tool execution.

### 3.6 Internationalization (i18n)
- **Multi-language Support**: The system shall be designed to support Internationalization (i18n), enabling localization for system messages, logs, and user-facing interactions.

### 3.7 CLI Experience
- **Startup Banner**: Display a branded ASCII banner with version and configuration info on startup.
- **Slash Commands**: Support for slash commands (e.g., `/exit`, `/new`) in the CLI chat interface.

### 3.8 Slash Command System
- **Interception Layer**: Parses user input starting with `/` before reaching the LLM.
- **Registry**: Supports both built-in Rust functions and custom template-based commands.
- **Extensibility**: Automatically loads custom commands from `.sisyphus/command/*.md`.
- **Templating**: Expands custom command arguments into prompt templates.

## 4. Technical Architecture

### 4.1 Technology Stack
- **Language**: Rust
- **Architecture**: Cargo Workspace (Monorepo)

### 4.2 Key Components (Rust Crates)
- `crates/core`: Core agent logic, session, and memory.
- `crates/common`: Shared utilities, traits, and event bus.
- `crates/tools`: Standard tool implementations (fs, shell).
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
  - **CLI**: A lightweight client consuming the core directly or via server.

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
