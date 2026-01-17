## 1. Create shared connection utility

- [ ] 1.1 Create `crates/cli-core/src/commands/connection.rs` module with:
  - `ConnectionContext` struct (wraps ServerManager, Client, ownership flag)
  - `setup_connection(attach_url: Option<String>, config_path: Option<String>) -> Result<ConnectionContext>` function
  - Abstract logic for choosing between ServerManager::start vs ServerManager::connect
  - Explicit `owns_server` flag to control lifecycle (stop on exit vs leave running)
- [ ] 1.2 Move connection setup logic from existing `chat.rs::attach()` into the new utility
- [ ] 1.3 Write unit tests for connection utility (local spawn, remote connect, error handling)

## 2. Create new command modules in cli-core

- [ ] 2.1 Create `crates/cli-core/src/commands/repl.rs` module with REPL logic using connection utility
- [ ] 2.2 Create `crates/cli-core/src/commands/tui.rs` module with TUI logic using connection utility
- [ ] 2.3 Create `crates/cli-core/src/commands/msg.rs` module for one-shot message functionality
- [ ] 2.4 Update `crates/cli-core/src/lib.rs` to export new modules (repl, tui, msg, connection)

## 3. Implement --attach flag support

- [ ] 3.1 Add `--attach <url>` optional argument to `repl` command clap struct
- [ ] 3.2 Add `--attach <url>` optional argument to `tui` command clap struct
- [ ] 3.3 Add `--attach <url>` optional argument to `msg` command clap struct
- [ ] 3.4 Wire up each command to use `setup_connection()` with appropriate attach_url

## 4. Implement one-shot message functionality

- [ ] 4.1 Implement message API call in `msg.rs` using connection utility
- [ ] 4.2 Handle message sending with streaming response collection
- [ ] 4.3 Implement graceful termination (stop owned server, print response, exit)
- [ ] 4.4 Add input validation (reject empty or whitespace-only messages)

## 5. Update CLI entry point

- [ ] 5.1 Refactor `crates/cli/src/main.rs` clap `Subcommand` enum:
  - Remove `Chat { tui: bool }`
  - Remove `Attach { url: String }`
  - Add `Repl { attach: Option<String> }`
  - Add `Tui { attach: Option<String> }`
  - Add `Msg { attach: Option<String>, message: String }`
  - Keep `Serve { port: u16 }` unchanged
- [ ] 5.2 Update command matching logic in `main()` to call new command modules
- [ ] 5.3 Update command help text and descriptions with examples

## 6. Clean up old code

- [ ] 6.1 Remove `crates/cli-core/src/commands/chat.rs` (functionality migrated to repl/tui/msg)
- [ ] 6.2 Remove `crates/cli-core/src/commands/attach.rs` (functionality migrated to connection.rs)
- [ ] 6.3 Verify no duplicate code remains between new modules
- [ ] 6.4 Update `crates/cli-core/src/commands/mod.rs` exports

## 7. Create design documentation

- [ ] 7.1 Create `openspec/changes/refactor-cli-command-structure/design.md` with:
  - Architecture diagrams showing module structure
  - Connection lifecycle decision tree
  - Server ownership and shutdown logic
  - Code organization rationale

## 8. Update spec deltas

- [ ] 8.1 Add error scenarios to each new requirement (connection failures, invalid URLs, timeouts)
- [ ] 8.2 Ensure spec deltas explicitly state implicit server spawning behavior
- [ ] 8.3 Verify all scenarios cover both success and error paths

## 9. Validation and testing

- [ ] 9.1 Manual test: Verify `sisyphus repl` starts local REPL and stops server on exit
- [ ] 9.2 Manual test: Verify `sisyphus tui` starts local TUI (with `--features tui`)
- [ ] 9.3 Manual test: Verify `sisyphus msg "hello"` sends one-shot message and terminates
- [ ] 9.4 Manual test: Verify `sisyphus serve` still works
- [ ] 9.5 Manual test: Verify `sisyphus repl --attach http://localhost:4000` connects without spawning
- [ ] 9.6 Manual test: Verify `sisyphus tui --attach http://localhost:4000` connects without spawning
- [ ] 9.7 Manual test: Verify `sisyphus msg --attach http://localhost:4000 "hello"` sends to remote
- [ ] 9.8 Error test: Verify malformed URL shows clear error message
- [ ] 9.9 Error test: Verify non-responsive server times out with helpful message
- [ ] 9.10 Error test: Verify empty message to `msg` command shows validation error
- [ ] 9.11 Verify `cargo build --release` succeeds
- [ ] 9.12 Verify `cargo test` passes (or fix any regressions)
- [ ] 9.13 Run `openspec validate refactor-cli-command-structure --strict`
