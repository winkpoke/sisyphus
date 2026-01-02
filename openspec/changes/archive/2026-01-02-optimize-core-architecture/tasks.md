# Tasks

## 1. Concurrency Refactor
- [x] Add `dashmap` to `crates/core/Cargo.toml`.
- [x] Refactor `SessionManager` struct to use `DashMap<String, Arc<RwLock<Session>>>`.
- [x] Update `SessionManager::create_session` to return `Arc<RwLock<Session>>`.
- [x] Update `SessionManager::get_session` to return `Option<Arc<RwLock<Session>>>`.
- [x] Remove `SessionManager::get_session_mut`.
- [x] Update `Server::chat` handler to acquire session lock locally instead of locking the manager.
- [x] Verify no deadlocks are introduced.

## 2. Command System Refactor
- [x] Create `crates/core/src/command/loader.rs`.
- [x] Move `load_from_dir` logic from `CommandRegistry` to `CommandLoader`.
- [x] Update `Agent` to use `CommandLoader` for initialization.
- [x] Add unit tests for `CommandLoader`.

## 3. API Optimization
- [x] Define `SessionSummary` struct in `crates/core/src/session/mod.rs` (or `dto.rs`).
- [x] Update `SessionManager::list_sessions` to return `Vec<SessionSummary>` (or keep it generic and map in the handler).
- [x] Update `Server::list_sessions` handler to return `Json<Vec<SessionSummary>>`.

## 4. Verification
- [x] Run `cargo test` to ensure no regressions.
- [x] Manual test: Start a long chat generation and try to list sessions in another terminal (should not block).
