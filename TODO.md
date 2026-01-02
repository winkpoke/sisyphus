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
