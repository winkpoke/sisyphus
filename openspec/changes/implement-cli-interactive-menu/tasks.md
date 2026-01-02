# Implementation Tasks

- [ ] **Backend (Server)**
    - [ ] Add `CommandInfo` struct (name, description, type) in `crates/core/src/command.rs`.
    - [ ] Implement `list_commands` in `Agent` struct.
    - [ ] Add `GET /api/v1/commands` route in `server/src/lib.rs`.
    - [ ] Verify endpoint returns correct JSON.

- [ ] **Client**
    - [ ] Add `get_commands` method to `Client` struct in `crates/client/src/client.rs`.
    - [ ] Update `MockProvider` if necessary for tests.

- [ ] **CLI**
    - [ ] Add `reedline` dependency to `crates/cli/Cargo.toml`.
    - [ ] Create `crates/cli/src/completer.rs` implementing `reedline::Completer`.
    - [ ] Refactor `run_chat` in `crates/cli/src/main.rs` to use `Reedline`.
    - [ ] Implement Keybinding: Bind `/` to `InsertChar('/')` + `Menu("completion_menu")`.
    - [ ] Configure `IdeMenu` with `name: "completion_menu"`.

- [ ] **Verification**
    - [ ] Launch `sisyphus serve`.
    - [ ] Launch `sisyphus chat` (or just `sisyphus`).
    - [ ] Press `/` and verify menu appears.
    - [ ] Navigate with arrows and select `help`.
    - [ ] Verify command executes.
