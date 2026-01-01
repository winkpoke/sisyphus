# Refactor CLI to Client-Server Architecture

## Summary
Refactor the Sisyphus CLI to operate as a thin client that connects to a server instance (either existing or spawned on demand), aligning with the OpenCode architecture pattern.

## Motivation
- **Consistency**: Matches OpenCode's established pattern for TUI/CLI tools.
- **Scalability**: Decouples the UI from the core logic, enabling remote access and multiple clients.
- **Resilience**: The server can persist independently of the CLI session.
- **Testability**: Easier to test the server and client components in isolation.

## Solution
1.  Introduce a `sisyphus-client` crate (or module within `cli`) to handle API communication.
2.  Update `sisyphus` to:
    -   Check for a running server or spawn a temporary one (headless).
    -   Connect via HTTP/SSE.
    -   Proxy user input to the server and stream responses back.
    -   Shutdown the temporary server on exit.

## Risks
- **Complexity**: Adds network layer overhead to local operations.
- **Performance**: Slight latency increase due to IPC/network stack (negligible for localhost).
- **Process Management**: Managing child processes (server) reliably across platforms (Windows/Linux) requires care (signal handling, zombie processes).
