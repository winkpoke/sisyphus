# Implementation Tasks

- [ ] **Shared Client Dispatch**
  - [ ] Add a shared slash-command router used by both REPL and TUI.
  - [ ] Ensure slash command identification uses the shared core parser.

- [ ] **Client-Local Command Registry**
  - [ ] Add a client-local command registry with consume/forward/pass outcomes.
  - [ ] Implement deterministic collision policy for local vs remote command names.

- [ ] **TUI Integration**
  - [ ] Route all input through the shared router.
  - [ ] Populate command palette with merged local + remote command lists.

- [ ] **REPL Integration**
  - [ ] Route all input through the shared router.
  - [ ] Keep behavior consistent with server lifecycle responses.

- [ ] **Verification**
  - [ ] In REPL and TUI, verify `/cmd "arg 1" arg2` routes consistently.
  - [ ] Verify a client-local command consumes locally without server call.
  - [ ] Verify a client-local command can forward a rewritten message to server.
  - [ ] Verify remote lifecycle effects (`/new`, `/clear`, `/exit`) behave identically across REPL and TUI.
  - [ ] Run workspace tests and ensure OpenSpec validation passes.

