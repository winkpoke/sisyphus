## 1. Core Command Runtime
- [x] Define command outcome/effect types in `crates/core/src/command.rs` <!-- id: 0 -->
- [x] Define async built-in command interface and argument parsing contract <!-- id: 1 -->
- [x] Refactor built-in command registry to store stateful commands <!-- id: 2 -->
- [x] Preserve custom markdown command loading and template expansion behavior <!-- id: 3 -->

## 2. Built-in Commands
- [x] Implement `/help` as a built-in command object <!-- id: 4 -->
- [x] Implement `/exit` and `/quit` as lifecycle commands <!-- id: 5 -->
- [x] Implement `/new` to request a new session ID (fresh session) <!-- id: 6 -->
- [x] Implement `/clear` to clear history for the current session ID <!-- id: 7 -->

## 3. Agent Integration
- [x] Update `Agent::chat` to return output plus lifecycle effect <!-- id: 8 -->

## 4. Server & Client Integration
- [x] Extend server chat response to include optional new session ID <!-- id: 9 -->
- [x] Update client to adopt new session IDs returned by chat responses <!-- id: 10 -->
- [x] Update CLI REPL to use the updated session ID for subsequent requests <!-- id: 11 -->

## 5. Validation
- [x] Add/update tests for `/new`, `/clear`, and `/exit` behavior across server/cli modes <!-- id: 12 -->
- [x] Run `openspec validate refactor-command-system --strict` <!-- id: 13 -->
