# Implement Server & API

## Summary
Implement the core HTTP and WebSocket server for Sisyphus using Axum, enabling the "Headless Agent" architecture defined in PRD Section 3.5. This server will expose session management, chat functionality, and real-time event streaming.

## Motivation
To support various clients (CLI, IDE extensions, Web UI) interacting with the Sisyphus agent, we need a robust API layer that decouples the core logic from the presentation. This implementation is critical for the "Headless Architecture" goal.

## Approach
- Use `axum` as the web framework.
- Implement `AppState` to share `Agent`, `SessionManager`, and `EventBus`.
- Expose REST endpoints for Session CRUD.
- Expose SSE/WebSocket for real-time events.
- Replicate key patterns from `opencode` (Hono-based) adapted for Rust/Axum.
