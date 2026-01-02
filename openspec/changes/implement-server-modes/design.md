# Design: Server Modes

## Context
The `sisyphus` binary serves two purposes:
1.  **Interactive CLI**: The user runs `sisyphus` (or `sisyphus chat`), which spawns a background server instance managed by the CLI process.
2.  **Background Service**: The user (or system) runs `sisyphus serve` to host the agent API for potentially multiple clients.

## Architecture

### Lifecycle Strategy Pattern

Instead of a boolean flag, we define a `LifecyclePolicy` strategy trait that decouples the server's runtime loop from deployment specifics.

```rust
pub trait LifecyclePolicy: Send + Sync {
    /// Determines action to take on global shutdown request
    fn on_shutdown_request(&self) -> ShutdownAction;
}

pub enum ShutdownAction {
    /// Proceed with graceful shutdown
    Proceed,
    /// Ignore the request (keep running)
    Ignore,
}
```

### Modes

| Mode | Policy Implementation | Trigger | Behavior on `Shutdown` Event |
|------|-----------------------|---------|------------------------------|
| **Standalone** | `StandalonePolicy` | `sisyphus serve --standalone` | **Proceed**: Log info, initiate graceful shutdown. |
| **Service** | `ServicePolicy` | `sisyphus serve` (default) | **Ignore**: Log warning/audit. Server continues running. |

### Session Ownership & Multi-Tenancy

To support secure multi-user environments (Service mode), the system must enforce session ownership.

#### Concept
*   **Client Identity**: Every connection or request context must identify a `owner_id` (e.g., "cli-user", "user-123"). In Standalone mode, this defaults to a single local user ID.
*   **Session Binding**: When a session is created, it is irrevocably bound to the `owner_id` of the creator.
*   **Authorization**: Operations like `EndSession` are validated against the session's owner.

#### Data Structures
The `SessionManager` tracks ownership:
*   `sessions: HashMap<SessionId, Session>` (Session struct now includes `owner_id: String`)
*   `client_sessions: HashMap<OwnerId, HashSet<SessionId>>` (Index for fast lookup/cleanup)

### Shutdown Protocol

We replace "Immediate Exit" with a **Fast Graceful Shutdown** to ensure data integrity:

1.  **Stop Acceptance**: Reject new requests/events.
2.  **Cancellation**: Signal cancellation tokens for running tasks.
3.  **Resource Cleanup**: 
    - Flush database WAL (if applicable).
    - Close file handles.
4.  **Exit**: Terminate process.

### Component Changes

#### 1. Common (`crates/common`)
- Update `SystemEvent` enum:
    - `EndSession { session_id: String, reason: Option<String> }`.
        - *Note*: The event implies "Request to End". Validation happens in the handler using context known to the Server/SessionManager.
    - `Shutdown` (No change).

#### 2. CLI (`crates/cli`)
- Update `Commands::Serve` to accept `--standalone`.
- In `server_manager`, instantiate `Server` with the appropriate `LifecyclePolicy`:
    - `--standalone` -> `Box::new(StandalonePolicy)`
    - Default -> `Box::new(ServicePolicy)`

#### 3. Server (`crates/server`)
- **Session Manager**:
    - Update `create_session` to accept `owner_id`.
    - Update `end_session` to accept `requester_id` and validate `session.owner_id == requester_id`.
- **Lifecycle**:
    - Introduce `lifecycle` module defining `LifecyclePolicy` trait and implementations.
    - Update `Server` struct to hold `Box<dyn LifecyclePolicy>`.
- **Event Loop (`Server::run`)**:
    - On `SystemEvent::Shutdown`:
        - Consult `policy.on_shutdown_request()`.
    - On `SystemEvent::EndSession`:
        - *Implementation Detail*: The `EndSession` event on the bus might need to carry `requester_id` if the bus is the only communication channel, OR the API layer validates before publishing. For this design, we assume the event contains the necessary authorization context or is trusted internal communication triggered by a validated API call.

### Security Considerations

- **Authorization**: `EndSession` requires ownership check. `SessionManager` enforces this.
- **Service Stability**: `ServicePolicy` prevents a single client from accidentally taking down the shared server via a `Shutdown` event.
- **Data Integrity**: Graceful shutdown ensures no partial writes or corrupted DB states, even in "fast" standalone exits.
