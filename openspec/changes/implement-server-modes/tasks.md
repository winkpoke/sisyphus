# Tasks

- [ ] Update `SystemEvent` in `crates/common/src/bus.rs` to include `EndSession` with `reason`. <!-- id: 8 -->
- [ ] Create `crates/server/src/lifecycle.rs` to define `LifecyclePolicy` trait and `ShutdownAction` enum. <!-- id: 9 -->
- [ ] Implement `StandalonePolicy` and `ServicePolicy` in `crates/server/src/lifecycle.rs`. <!-- id: 10 -->
- [ ] Update `Server` struct in `crates/server/src/lib.rs` to hold `Box<dyn LifecyclePolicy>`. <!-- id: 4 -->
- [ ] Implement `perform_graceful_shutdown` in `Server` (flush, cancel, exit). <!-- id: 11 -->
- [ ] Update `Server::run` in `crates/server/src/lib.rs` to use policy for `Shutdown` events. <!-- id: 5 -->
- [ ] Add `standalone` flag to `Commands::Serve` in `crates/cli/src/main.rs`. <!-- id: 1 -->
- [ ] Update `ServerManager::start` in `crates/cli/src/server_manager.rs` to inject the correct policy based on flag. <!-- id: 3 -->
- [ ] Verify `sisyphus chat` (Standalone) exits cleanly on shutdown command. <!-- id: 6 -->
- [ ] Verify `sisyphus serve` (Service) logs warning and ignores shutdown command. <!-- id: 7 -->
- [ ] Update `Session` struct to include `owner_id`. <!-- id: 12 -->
- [ ] Update `SessionManager` to index sessions by `owner_id` and validate ownership in `end_session`. <!-- id: 13 -->
