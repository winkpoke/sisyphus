# Optimize Core Architecture

## Summary
This change addresses critical performance bottlenecks and architectural issues in the core system. It introduces granular locking for session management to prevent server-wide blocking during LLM inference, refactors the command system to separate concerns, and optimizes API endpoints for better memory usage.

## Motivation
1.  **Critical Lock Contention**: Currently, `SessionManager` uses a single lock. During `agent.chat`, this lock is held for the duration of the inference (which can take seconds), blocking all other server operations.
2.  **Architectural Coupling**: `CommandRegistry` handles both command storage and file I/O, violating the Single Responsibility Principle.
3.  **Performance**: List endpoints clone entire session histories, leading to unnecessary memory allocation and serialization overhead.

## Proposed Solution
1.  **Granular Concurrency**: Refactor `SessionManager` to use `DashMap` and `Arc<RwLock<Session>>`.
2.  **Command Refactor**: Extract file loading logic into a dedicated `CommandLoader`.
3.  **DTOs**: Implement lightweight Data Transfer Objects for list endpoints.
