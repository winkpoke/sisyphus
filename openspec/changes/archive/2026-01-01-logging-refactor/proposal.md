# Logging Refactor: Decouple Event Logging

## Context
Currently, the `main.rs` file directly handles the subscription to the `EventBus` and prints events to the console using `println!`. This violates the Single Responsibility Principle and clutters the entry point of the application.

## Goals
- Move event logging logic out of `main.rs`.
- Implement a dedicated `start_event_logger` function in `common/logging.rs`.
- Use `tracing` for structured logging of system events instead of `println!`.
- Ensure `main.rs` only initializes the logger.

## Architecture Highlights
- **Event Logger**: A background task that subscribes to the `EventBus` and logs events using `tracing`.
- **Main Loop**: The main application loop remains focused on initialization and running the agent.
