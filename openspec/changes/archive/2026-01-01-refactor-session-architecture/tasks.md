# Tasks

- [x] Define `SessionStatus` enum (Idle, Busy) in `crates/core/src/session.rs` <!-- id: 0 -->
- [x] Create `SessionManager` struct in `crates/core/src/session/manager.rs` with `create`, `get`, `list` methods <!-- id: 1 -->
- [x] Refactor `Session` struct to include `id` and `status` <!-- id: 2 -->
- [x] Refactor `Agent::chat` to accept `&mut Session` instead of using `self.session` <!-- id: 3 -->
- [x] Update `Agent` struct to remove `session` field (making it stateless/config-only) <!-- id: 4 -->
- [x] Update `crates/cli/src/main.rs` to initialize `SessionManager`, create a default session, and pass it to `Agent` loop <!-- id: 5 -->
- [x] Verify `Session` locking mechanism (prevent concurrent `chat` calls on same session) <!-- id: 6 -->
