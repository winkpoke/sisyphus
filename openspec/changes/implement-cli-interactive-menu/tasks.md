# Implementation Tasks

- [x] **Backend (Server)**
    - [x] Add `CommandInfo` struct (name, description, type) in `crates/core/src/command.rs`.
    - [x] Implement `list_commands` in `Agent` struct.
    - [x] Add `GET /api/v1/commands` route in `server/src/lib.rs`.
    - [x] Verify endpoint returns correct JSON.

- [x] **Client**
    - [x] Add `get_commands` method to `Client` struct in `crates/client/src/client.rs`.
    - [x] Update `MockProvider` if necessary for tests. (N/A)

- [x] **CLI**
    - [x] Add `reedline` dependency to `crates/cli/Cargo.toml`.
    - [x] Create `crates/cli/src/completer.rs` implementing `reedline::Completer`.
    - [x] Refactor `run_chat` in `crates/cli/src/main.rs` to use `Reedline`.
    - [x] Implement Keybinding: Bind `/` to `InsertChar('/')` + `Menu("completion_menu")`.
    - [x] Configure `IdeMenu` (used `ColumnarMenu`) with `name: "completion_menu"`.

- [ ] **Verification**
    - [ ] Launch `sisyphus serve`.
    - [ ] Launch `sisyphus chat` (or just `sisyphus`).
    - [ ] Press `/` and verify menu appears.
    - [ ] Navigate with arrows and select `help`.
    - [ ] Verify command executes.
