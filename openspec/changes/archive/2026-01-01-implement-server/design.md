# Server Architecture Design

## Overview
The server acts as the interface layer between the Core Agent and external clients. It manages state synchronization and exposes capabilities via HTTP and Events.

## Components

### 1. Web Framework (Axum)
We choose `axum` for its strong integration with `tokio` and `tower` ecosystem.

### 2. State Management
The `AppState` struct will hold `Arc` references to core components:
```rust
struct AppState {
    agent: Arc<Agent>,
    session_manager: Arc<tokio::sync::Mutex<SessionManager>>,
    bus: Arc<EventBus>,
}
```

### 3. API Surface
#### Sessions
- `GET /api/v1/sessions`: List active sessions.
- `POST /api/v1/sessions`: Create a new session.
- `GET /api/v1/sessions/:id`: Get session details/history.
- `POST /api/v1/sessions/:id/chat`: Send a message (Supports Streaming).

#### Events
- `GET /api/v1/events`: Server-Sent Events (SSE) stream for `SystemEvent`.

## Opencode Comparison
| Feature | Opencode (Node/Hono) | Sisyphus (Rust/Axum) |
| :--- | :--- | :--- |
| Framework | Hono | Axum |
| Events | SSE (`/global/event`) | SSE (`/api/v1/events`) |
| Session | REST CRUD | REST CRUD |
| Chat | `POST /session/:id/message` (Stream) | `POST /api/v1/sessions/:id/chat` (Stream) |

## Security
- Basic CORS configuration initially.
- No authentication for this phase (local execution assumed).
