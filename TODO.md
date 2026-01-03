# Sisyphus Refactoring & Improvement Plan

## 1. Architecture & Separation of Concerns

### Core Refactoring (Hexagonal Architecture)
- [ ] **Define Domain Layer**: Create `crates/core/src/domain` for pure entities (`Session`, `Message`) with no external dependencies.
- [ ] **Define Ports**: Create `crates/core/src/ports` for interfaces (`SessionRepository`, `LLMProvider`).
- [ ] **Implement Service Layer**: Create `crates/core/src/services` to encapsulate business logic (`ChatService`, `SessionService`).
  - `ChatService` should handle the flow: Retrieve Session -> Call Agent -> Save Result.
- [ ] **Remove Circular/Direct Dependencies**: Refactor `server` crate to depend on `Service` layer, not internal `SessionManager` or `Agent` logic.

### Dependency Injection
- [ ] Introduce a proper DI container or manual composition root in `main.rs` to wire Repositories -> Services -> Handlers.

## 2. Security & Performance

### Concurrency & Locking
- [ ] **Remove Global Mutex**: Replace global `Arc<Mutex<SessionManager>>` in `server/src/lib.rs` with fine-grained locking (e.g., `DashMap` or `RwLock` inside the Repository implementation).
- [ ] **Async/Await Correctness**: Ensure no blocking operations exist within async handlers.

### Data Persistence
- [ ] **Abstract Storage**: Move away from in-memory-only `SessionManager`. Implement `SessionRepository` trait.
- [ ] **Implement Persistence Adapter**: Create a file-based or database-backed implementation of `SessionRepository` to prevent data loss on restart.

### Optimization
- [ ] **Reduce Cloning**: Optimize `Agent::process_turn` (crates/core/src/agent.rs) to avoid cloning the entire session history on every loop iteration.
- [ ] **Memory Management**: Review large object passing (e.g., history vectors) and pass by reference where possible.

## 3. Code Quality & Standards

### Error Handling
- [ ] **Typed Errors**: Replace `(StatusCode, String)` tuples in `server` handlers with a proper `AppError` enum that implements `IntoResponse`.
- [ ] **Result Propagation**: Ensure `anyhow::Result` is used consistently in `core` and mapped correctly at the boundary.

### Clean Code
- [ ] **Remove Magic Strings**: Replace hardcoded strings like "user", "assistant", event types with `enum` constants.
- [ ] **Immutability**: Refactor `Agent::chat` to avoid taking `&mut Session` if possible, or clearly separate state mutation from logic generation.

### Testing
- [ ] **Unit Tests**: Add tests for `ChatService` using mock Repositories.
- [ ] **Integration Tests**: Verify the `server` endpoints using the new architecture.

## 4. Immediate Refinements (Post-Refactor)

### Session Management
- [ ] **Encapsulate Session Locking**: Refactor `SessionManager::list_sessions` to return `Vec<SessionSummary>` directly.
  - *Current State*: Returns `Vec<Arc<RwLock<Session>>>`, exposing locks to the caller and risking "lock storms".
  - *Goal*: Handle locking internally within `SessionManager` and return pure DTOs.

### Command System
- [ ] **Robust Command Loading**: Update `CommandLoader` to return a `LoadResult` with both successes and errors.
  - *Current State*: Swallows errors (logs warning) for individual files during loading.
  - *Goal*: Allow the application to report all loading failures to the user/admin.
- [ ] **Strict Name Validation**: Enforce strict alphanumeric naming for commands in `CommandLoader`.
  - *Goal*: Prevent security issues and ensure compatibility with future filesystem/API mappings.

## 5. Critical Bugs
- [ ] **Memory Leak in Session Manager**
  - **Location**: `crates/core/src/session/manager.rs`
  - **Issue**: `SessionManager` stores sessions in a `DashMap` but provides no mechanism to remove them. `CommandEffect::NewSession` creates a new session but leaves the old one in the map.
  - **Impact**: Long-running servers will eventually run out of memory as abandoned sessions accumulate.
  - **Fix**: Implement a cleanup strategy (e.g., TTL, explicit `delete_session`, or LRU eviction).

## 6. CLI/TUI Improvements
- [ ] **Monolithic Tui::run Refactor**
  - **Location**: `crates/cli/src/ui/tui/mod.rs`
  - **Issue**: `run` method is ~450 lines mixing layout, logic, and event handling.
  - **Goal**: Split into `draw_ui`, `handle_input`, and `handle_events`.
- [ ] **Async Task Error Handling**
  - **Location**: `crates/cli/src/ui/tui/mod.rs` (background tasks)
  - **Issue**: `let _ = tx.send(...)` swallows errors. If the receiver dies, the UI freezes without feedback.
  - **Goal**: Log errors or show a UI alert when channel sending fails.
- [ ] **Secure Redaction**
  - **Location**: `crates/cli/src/ui/tui/mod.rs` (`redact_json`)
  - **Issue**: Falls back to printing raw string if JSON parse fails, potentially leaking secrets.
  - **Goal**: Use regex-based fallback or conservative masking for non-JSON payloads.
