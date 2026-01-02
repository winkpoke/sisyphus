# Architecture Design

## 1. Concurrency Model

### Problem
The current `SessionManager` implementation wraps a `HashMap<String, Session>` behind a single `Mutex` in the `AppState`.
```rust
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}
// Usage in Server
session_manager: Arc<Mutex<SessionManager>>
```
When `agent.chat` is called, it locks the manager to get a mutable reference to the session. This lock is held throughout the async `chat` call, which includes LLM inference.

### Solution
We will move to a concurrent map structure where the manager itself doesn't need to be locked for individual session access.
```rust
pub struct SessionManager {
    // DashMap handles concurrent access to the map itself
    sessions: DashMap<String, Arc<RwLock<Session>>>,
}
```
*   **Reads**: `get_session` returns `Arc<RwLock<Session>>`. The caller acquires a read lock.
*   **Writes**: `get_session_mut` is deprecated/removed in favor of internal mutability via the `RwLock`.
*   **Chat**: The `chat` method will take `Arc<RwLock<Session>>`, acquiring a write lock only on the specific session, leaving the manager free for other operations.

## 2. Command System

### Problem
`CommandRegistry` currently mixes two concerns:
1.  Runtime registry of available commands (`HashMap<String, CommandType>`).
2.  File system scanning and parsing (`load_from_dir`).

### Solution
Split into:
*   `CommandRegistry`: Pure in-memory container.
*   `CommandLoader`: Stateless service/function responsible for scanning directories, parsing Markdown/Frontmatter, and returning `CommandConfig` objects.

## 3. API Optimization

### Problem
`GET /api/v1/sessions` returns `Vec<Session>`. Since `Session` contains the full `Vec<Message>`, this payload grows linearly with conversation history size.

### Solution
Introduce `SessionSummary` DTO:
```rust
#[derive(Serialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at: DateTime<Utc>, // If available, or add it
    pub message_count: usize,
    pub status: SessionStatus,
}
```
The list endpoint will map sessions to this summary type.
