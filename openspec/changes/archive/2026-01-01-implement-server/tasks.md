# Implementation Tasks

- [x] Add dependencies to `crates/server/Cargo.toml` (axum, tower, etc.) <!-- id: deps -->
- [x] Implement `AppState` and Server struct in `crates/server/src/lib.rs` <!-- id: server_struct -->
- [x] Implement `GET /health` endpoint <!-- id: health_endpoint -->
- [x] Implement `SessionManager` thread-safe wrapper <!-- id: session_wrapper -->
- [x] Implement `GET /api/v1/sessions` and `POST /api/v1/sessions` <!-- id: session_crud -->
- [x] Implement `GET /api/v1/events` (SSE) <!-- id: sse_endpoint -->
- [x] Implement `POST /api/v1/sessions/:id/chat` <!-- id: chat_endpoint -->
- [x] Verify server with `curl` or simple test script <!-- id: verify -->
