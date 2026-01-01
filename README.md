# Sisyphus

<p align="center">
  <strong>A robust, open-source general-purpose AI agent framework</strong>
</p>

<p align="center">
  <a href="LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square">
  </a>
  <a href="https://rust-lang.org">
    <img alt="Rust" src="https://img.shields.io/badge/rust-1.75+-orange.svg?style=flat-square">
  </a>
</p>

---

## 📖 About

**Sisyphus** is a general-purpose AI agent framework ported from the core logic of [OpenCode](https://opencode.ai). It is designed to be a flexible, headless, and extensible system for building AI agents that can interact with the world through tools and APIs.

> **Note**: This project is currently in early active development (Phase 1).

### Core Principles

1.  **Provider Agnostic** - Support for multiple LLM providers (OpenAI implemented, others planned).
2.  **Extensible** - Universal tooling system.
3.  **Headless by Design** - Decoupled architecture allowing multiple client interfaces.

## 🏗️ Architecture

Sisyphus uses a **Cargo Workspace** architecture:

```
sisyphus/
├── crates/
│   ├── common/      # Shared utilities, traits, event bus
│   ├── core/        # Agent logic, session management
│   ├── provider/    # LLM provider adapters
│   ├── tools/       # Standard tools (fs, shell)
│   ├── server/      # HTTP/WebSocket API server (Planned)
│   └── cli/         # Command-line interface
└── Cargo.toml       # Workspace configuration
```

## ✨ Features

### Current Capabilities

-   **Interactive CLI**: Chat with the agent directly in your terminal.
-   **LLM Support**:
    -   OpenAI (GPT-4, etc.)
    -   Mock Provider (for testing)
-   **Tools**:
    -   **Shell Execution**: Run system commands.
    -   **File System**: Read and write files within a sandboxed environment.
-   **Event System**: Internal event bus for observability.

### Planned Features

-   **MCP Support**: Native integration with Model Context Protocol.
-   **Server API**: HTTP/WebSocket server for remote clients.
-   **More Providers**: Anthropic, Google, Local Models (Ollama/vLLM).
-   **Specialized Agents**: Planner/Executor separation.

## 🚀 Quick Start

### Prerequisites

-   Rust 1.75 or higher
-   Cargo package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/sisyphus.git
cd sisyphus

# Build the project
cargo build --release
```

### Configuration

Create a `.env` file in the project root:

```env
# LLM Provider Configuration
LLM_PROVIDER=openai
OPENAI_API_KEY=your-api-key

# Optional
LLM_MODEL=gpt-4
# LLM_BASE_URL=...
```

### Usage

Run the CLI in chat mode:

```bash
cargo run --release --bin sisyphus
```

Or if installed:

```bash
sisyphus chat
```

## � Documentation

-   **[PRD.md](PRD.md)** - Product requirements and feature specifications.
-   **[AGENTS.md](AGENTS.md)** - Development guidelines and agent instructions.

## �🛠️ Development

### Building and Testing

```bash
# Build all crates
cargo build

# Run all tests
cargo test
```

### Adding a New Tool

1.  Implement the `Tool` trait in `crates/tools/src/`.
2.  Register the tool in `crates/tools/src/lib.rs`.
3.  Add it to the agent in `crates/cli/src/main.rs`.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

-   **OpenCode** - Core logic and inspiration.
-   **Rust Community** - For the amazing ecosystem.
