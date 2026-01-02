# Tasks: Refactor CLI to Client-Server

- [x] Create `crates/client` library @high
  - [x] Implement `Client` struct with `reqwest`.
  - [x] Implement `health_check`, `create_session`, `chat` methods.
  - [x] Implement `subscribe_events` (SSE) using `reqwest-eventsource`.
- [x] Refactor `crates/cli` to use `crates/client` @high
  - [x] Remove direct `Agent` and `SessionManager` usage from `run_chat`.
  - [x] Implement `ServerManager` struct to handle spawning/killing server process.
  - [x] Implement `run_chat` using `Client` SDK.
  - [x] Handle Ctrl+C to ensure server cleanup.
- [x] Verify End-to-End @high
  - [x] Test `sisyphus` spawns server and works.
  - [x] Test `sisyphus attach` works with existing server.
  - [x] Verify server process is killed on exit.
