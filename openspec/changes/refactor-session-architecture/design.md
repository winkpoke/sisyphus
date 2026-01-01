## Context
We are aligning Sisyphus with OpenCode's architecture where "Session" is the runtime container (State) and "Agent" is the configuration (Behavior). Currently, Sisyphus combines them.

## Goals
- **Decoupling**: `Session` should not know about `Agent` logic. `Agent` should not own `Session` data.
- **Multi-Session Support**: The system must support multiple active sessions via `SessionManager`.
- **Persistence Ready**: The architecture should facilitate saving/loading sessions from disk (future work).

## Design

### 1. Session Manager
A central struct to hold active sessions.
```rust
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    // active_session_id: Option<String>, // For CLI focus
}
```

### 2. Session Entity
Holds the "State".
```rust
pub struct Session {
    pub id: String,
    pub history: Vec<Message>,
    pub status: SessionStatus, // Idle, Busy, etc.
}
```

### 3. Agent Execution (The Runner)
Instead of `agent.chat(input)`, we move to a pattern where the Agent *acts upon* a session.

```rust
impl Agent {
    // Stateless execution
    pub async fn run_turn(&self, session: &mut Session, input: &str) -> Result<String> {
        // 1. Add User Message to Session
        // 2. Build Context (System Prompt + History)
        // 3. Call LLM
        // 4. Handle Tool Calls (Loop)
        // 5. Add Assistant Message to Session
    }
}
```
Or, if we keep `Agent` as the high-level facade, it holds a reference to the `SessionManager`?
Better: `Agent` is just the *Configuration* and *Logic*. The CLI drives the interaction.

### 4. Concurrency Model
- **User Input**: Locked per session. If `session.status == Busy`, reject input.
- **Tool Execution**: Allowed to be parallel (future optimization), but `run_turn` is async and awaits completion.

## Migration Path
1.  Create `SessionManager`.
2.  Extract `Session` from `Agent`.
3.  Update CLI to create a Manager, create a Session, and pass the Session to the Agent for each turn.
