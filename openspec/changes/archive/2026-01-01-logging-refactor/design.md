# Design: Event Logging Decoupling

## Problem
The `main.rs` file contains logic for subscribing to the event bus and printing logs. This makes `main.rs` harder to read and maintain, and it mixes UI/CLI concerns with logging concerns.

## Solution
We will introduce a `start_event_logger` function in the `common` crate. This function will:
1.  Take a reference to the `EventBus`.
2.  Subscribe to the bus.
3.  Spawn a Tokio task to listen for events.
4.  Log each event using the `tracing` crate (e.g., `info!`, `debug!`).

## Trade-offs
- **Pros**: Cleaner `main.rs`, consistent logging via `tracing`, better separation of concerns.
- **Cons**: Adds a slight indirection, but this is standard for modular architectures.
