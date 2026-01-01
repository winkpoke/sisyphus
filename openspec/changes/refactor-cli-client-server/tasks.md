# Tasks: Refactor CLI to Client-Server

- [ ] Create `crates/client` library @high
  - [ ] Implement `Client` struct with `reqwest`.
  - [ ] Implement `health_check`, `create_session`, `chat` methods.
  - [ ] Implement `subscribe_events` (SSE) using `reqwest-eventsource`.
- [ ] Refactor `crates/cli` to use `crates/client` @high
  - [ ] Remove direct `Agent` and `SessionManager` usage from `run_chat`.
  - [ ] Implement `ServerManager` struct to handle spawning/killing server process.
  - [ ] Implement `run_chat` using `Client` SDK.
  - [ ] Handle Ctrl+C to ensure server cleanup.
- [ ] Verify End-to-End @high
  - [ ] Test `sisyphus` spawns server and works.
  - [ ] Test `sisyphus attach` works with existing server.
  - [ ] Verify server process is killed on exit.
