# Implement Server Modes (Standalone vs Service)

## Summary
This proposal introduces a robust architectural distinction between "Standalone" and "Service" modes for the Sisyphus server using a Strategy pattern. This distinction allows the application to optimize its behavior for single-user CLI interactions (fast but safe shutdown) versus multi-user background service operations (resilient uptime, explicit session management).

## Motivation
Currently, the Sisyphus server runs with identical logic regardless of how it was started. This leads to suboptimal user experiences and security risks:
- **CLI Usage**: Users expect immediate exit when quitting the CLI, but the server might linger or corrupt data if killed forcefully.
- **Service Usage**: A single user quitting their session (triggering a Shutdown event) kills the entire server for all users. This is a critical stability issue for multi-user deployments.
- **Maintainability**: Hardcoding boolean flags for modes leads to brittle code as requirements evolve.

## Proposed Changes
1.  **Architecture**: Implement a `LifecyclePolicy` strategy pattern to decouple server logic from deployment modes.
2.  **CLI**: Add a `--standalone` flag to the `serve` command which injects the `StandalonePolicy`.
3.  **Server**: Implement policy-driven event handling.
    - **Standalone**: Handle `Shutdown` by initiating a graceful shutdown sequence (flush, cancel, exit).
    - **Service**: Ignore `Shutdown` events, logging a warning to prevent accidental denial of service.
4.  **Events**: Update `EndSession` to include `reason` for better auditing and control.
