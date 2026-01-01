# Scaffold Phase 1: Foundation & Core Infrastructure

## Context
This proposal establishes the initial architectural groundwork for the `sisyphus` project, a Rust port of the core `opencode` logic. The goal is to ensure modularity, testability, and async capability suitable for AI agents.

## Goals
- Establish a Cargo Workspace structure.
- Implement core infrastructure (Config, Logging, Event Bus).
- Create a unified LLM Provider abstraction.
- Define data models and tool interfaces.
- Deliver a working "Hello World" that connects to an LLM.

## Architecture Highlights
- **Workspace**: Split into `common`, `provider`, `core`, `server`, `cli`, and `tools`.
- **Common**: Holds shared types, traits (`LLMProvider`), and utilities to prevent circular dependencies.
- **Provider**: Implements specific LLM adapters (OpenAI, Anthropic).
- **Core**: Contains pure domain logic (Agent, Session, Memory).
- **Tools**: Dedicated crate for tool definitions and implementations (FS, Shell), decoupled from Core and Provider.
- **Event Bus**: Dual-channel approach (`broadcast` for UI/Observability, `mpsc` for Agent Control Flow).
