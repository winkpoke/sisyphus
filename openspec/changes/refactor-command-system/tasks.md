## 1. Core Command Runtime
- [ ] Define command outcome/effect types in `crates/core/src/command.rs` <!-- id: 0 -->
- [ ] Define async built-in command interface and argument parsing contract <!-- id: 1 -->
- [ ] Refactor built-in command registry to store stateful commands <!-- id: 2 -->
- [ ] Preserve custom markdown command loading and template expansion behavior <!-- id: 3 -->

## 2. Built-in Commands
- [ ] Implement `/help` as a built-in command object <!-- id: 4 -->
- [ ] Implement `/exit` and `/quit` as lifecycle commands <!-- id: 5 -->
- [ ] Implement `/new` to request a new session ID (fresh session) <!-- id: 6 -->
- [ ] Implement `/clear` to clear history for the current session ID <!-- id: 7 -->

## 3. Agent Integration
- [ ] Update `Agent::chat` to return output plus lifecycle effect <!-- id: 8 -->

## 4. Server & Client Integration
- [ ] Extend server chat response to include optional new session ID <!-- id: 9 -->
- [ ] Update client to adopt new session IDs returned by chat responses <!-- id: 10 -->
- [ ] Update CLI REPL to use the updated session ID for subsequent requests <!-- id: 11 -->

## 5. Validation
- [ ] Add/update tests for `/new`, `/clear`, and `/exit` behavior across server/cli modes <!-- id: 12 -->
- [ ] Run `openspec validate refactor-command-system --strict` <!-- id: 13 -->
