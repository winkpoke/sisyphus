# Design: CLI Command Structure Refactor

## Context

The current CLI structure mixes interface selection (REPL vs TUI) with connection mode (local vs remote) in a way that's not intuitive. Users must remember three different commands (`chat`, `attach`, `serve`) for what are essentially two operations (interactive vs dedicated server) with optional remote connection.

**Current structure:**
```bash
sisyphus chat          # Local REPL
sisyphus chat --tui    # Local TUI (via flag)
sisyphus attach <url>  # Remote REPL (explicit command)
sisyphus serve          # Dedicated server
```

**Proposed structure:**
```bash
sisyphus repl          # Local REPL
sisyphus tui           # Local TUI (explicit command)
sisyphus msg "text"    # One-shot message
sisyphus repl --attach <url>  # Remote REPL
sisyphus serve          # Dedicated server (unchanged)
```

## Goals / Non-Goals

### Goals
1. **Clearer command semantics**: Explicit interface choice (repl/tui/msg) before connection mode
2. **Better composability**: `--attach` flag works with any interface
3. **Reduced mental model**: Two choices (interface × connection mode) instead of three commands
4. **Code organization**: Modular structure without duplication
5. **Explicit server ownership**: Clear lifecycle (local = own it, remote = don't own it)

### Non-Goals
1. **Backward compatibility**: This is a breaking change (hard break)
2. **Feature flag for TUI**: Keep existing `--features tui` compile-time flag
3. **Session management changes**: No changes to session creation/management APIs
4. **Permission system changes**: No changes to tool approval or permission flows

## Decisions

### Decision 1: Explicit Interface Commands

**Choice**: Create separate `repl`, `tui`, `msg` commands instead of `chat --tui` flag.

**Rationale**:
- Clearer semantics: Command name = interface type
- Better composability: `--attach` flag works uniformly
- Aligns with user mental model (think "I want TUI", then "attach to remote")
- Removes ambiguity about what `--tui` means (is it a mode? a flag? a command?)

**Alternatives considered**:
1. Keep `chat` with subcommands: `chat repl`, `chat tui`, `chat msg` → Too verbose
2. Keep `chat --tui` flag, add `--attach`: `chat --tui --attach <url>` → Confusing order of flags
3. Single command with mode arg: `sisyphus --mode=tui --attach=<url>` → Unclear primary vs optional args

### Decision 2: Shared Connection Utility

**Choice**: Create `connection.rs` utility with `setup_connection()` function.

**Rationale**:
- Avoids code duplication across 3 commands
- Centralizes complex lifecycle logic (start vs connect, own vs don't own)
- Makes testing easier (single unit test covers all commands)
- Consistent error handling for all connection modes

**Architecture**:
```rust
// crates/cli-core/src/commands/connection.rs
pub struct ConnectionContext {
    pub server_manager: ServerManager,
    pub client: Client,
    pub owns_server: bool,  // Should we call stop() on exit?
}

pub async fn setup_connection(
    attach_url: Option<String>,
    config_path: Option<String>,
) -> Result<ConnectionContext> {
    if let Some(url) = attach_url {
        // Remote: connect, don't own
        let server_manager = ServerManager::connect(parse_url(url)?).await?;
        Ok(ConnectionContext {
            server_manager,
            client: server_manager.client(),
            owns_server: false,
        })
    } else {
        // Local: spawn, own it
        let port = find_available_port()?;
        let server_manager = ServerManager::start(port, config_path).await?;
        Ok(ConnectionContext {
            server_manager,
            client: server_manager.client(),
            owns_server: true,
        })
    }
}
```

**Usage in commands**:
```rust
// repl.rs, tui.rs, msg.rs all use same pattern:
let ctx = setup_connection(attach_url, config_path).await?;
let mut repl = Repl::new(ctx.client, session_id, ctx.shutdown_rx);
repl.run().await?;

// After REPL/loop exits:
if ctx.owns_server {
    ctx.server_manager.stop().await?;
}
```

**Alternatives considered**:
1. Inline in each command → Violates "no duplicate code" requirement
2. Trait-based abstraction → Overkill for simple connection logic
3. Configuration-driven → More complex, no clear benefit

### Decision 3: Server Ownership Model

**Choice**: Explicit `owns_server` flag to control lifecycle (stop on exit vs leave running).

**Rationale**:
- Local server: CLI spawned it, so CLI must stop it (cleanup)
- Remote server: CLI doesn't own it, so leave it running
- Prevents accidental shutdown of shared remote servers
- Makes lifecycle explicit in type system, not implicit in code flow

**Lifecycle diagram**:
```
sisyphus repl (no --attach)
  └─> spawn local server [owns_server = true]
      └─> connect
      └─> run REPL
      └─> user exits
      └─> call server_manager.stop() ✓
      └─> exit

sisyphus repl --attach http://localhost:4000
  └─> connect to remote [owns_server = false]
      └─> run REPL
      └─> user exits
      └─> skip server_manager.stop() (not owned)
      └─> exit (server continues running)
```

**Edge cases**:
- If server fails during startup, `Drop` impl still kills process (as safety net)
- If `--attach` fails (malformed URL, unreachable), no server is spawned, no cleanup needed
- If user Ctrl+C during startup, `Drop` handles cleanup

### Decision 4: One-Shot Message Command

**Choice**: Add new `msg` command for non-interactive use.

**Rationale**:
- Scripts and automation need one-shot messages (CI/CD, Git hooks, cron)
- REPL is overkill for simple queries (startup overhead, interactive prompt)
- Aligns with `--attach` pattern (works with local and remote)
- Clear exit semantics (send, receive, exit)

**Behavior**:
1. Spawn/connect to server (via `setup_connection`)
2. Create session
3. Send message
4. Wait for complete response (collect streaming chunks)
5. Print response
6. Clean up (stop server if owned)
7. Exit with status code 0

**Error handling**:
- Empty/whitespace message: Validation error, exit non-zero, no server spawn
- Connection failure: Error message, exit non-zero
- Streaming timeout: Wait indefinitely (current) or timeout (future enhancement)

**Alternatives considered**:
1. `echo "hello" | sisyphus chat` → Pipe-based, less explicit
2. `sisyphus chat --message "hello"` → Still has REPL overhead
3. No one-shot command → Forces automation to use REPL, inefficient

### Decision 5: Module Structure

**Choice**: Organize commands by interface type in separate modules.

**Structure**:
```
crates/cli-core/src/commands/
├── connection.rs    # NEW: shared connection utility
├── repl.rs          # NEW: REPL command
├── tui.rs           # NEW: TUI command
├── msg.rs           # NEW: one-shot message command
├── serve.rs         # EXISTING: dedicated server (unchanged)
└── mod.rs           # Update exports
```

**Rationale**:
- Each file is single-purpose (one command per module)
- Easy to navigate and maintain
- Clear separation of concerns
- Testable in isolation

**Code flow**:
```
main.rs (CLI entry point)
  └─> parse args with clap
  └─> match command
      ├─> Repl { attach } ──> commands::repl::run(attach, config)
      ├─> Tui { attach } ──> commands::tui::run(attach, config)
      ├─> Msg { attach, message } ──> commands::msg::run(attach, message, config)
      └─> Serve { port } ──> commands::serve::run(config, port)
          └─> All use commands::connection::setup_connection()
```

## Risks / Trade-offs

### Risk 1: Breaking Existing User Workflows

**Risk**: Scripts, aliases, and documentation using `chat` or `attach` will break.

**Mitigation**:
- Clear migration guide in proposal.md
- Update all official documentation
- Provide migration examples in README
- Consider deprecation warning period (rejected: hard break is cleaner)

**Acceptable because**:
- Project is in early active development (Phase 1)
- User base is small (pre-1.0)
- Long-term clarity outweighs short-term friction

### Risk 2: Increased Code Complexity

**Risk**: Adding shared connection utility adds abstraction layer.

**Mitigation**:
- Keep utility simple (one function, one struct)
- Write clear documentation in code
- Add unit tests for all scenarios

**Acceptable because**:
- Reduces duplication by 2/3 (was 3 separate implementations)
- Centralizes complex lifecycle logic
- Makes future changes easier (e.g., add connection pooling)

### Risk 3: One-Shot Message Timeout Ambiguity

**Risk**: `msg` command may hang if server never responds.

**Mitigation**:
- Document current behavior (no timeout)
- Add issue tracking for configurable timeout
- Provide Ctrl+C support for manual cancellation

**Acceptable because**:
- Current REPL also has no timeout
- Simple implementation first, add timeout later if needed
- User can always Ctrl+C

## Migration Plan

### Steps

1. **Create new command modules** (parallel to old ones)
   - Don't remove `chat.rs` or `attach.rs` yet
   - Implement `connection.rs`, `repl.rs`, `tui.rs`, `msg.rs`
   - Add new clap subcommands alongside old ones

2. **Update main.rs to support both structures** (temporary)
   ```rust
   // During migration, support both:
   enum Commands {
       Chat { tui: bool },      // OLD (deprecated)
       Attach { url: String },    // OLD (deprecated)
       Repl { attach: Option<String> },  // NEW
       Tui { attach: Option<String> },   // NEW
       Msg { attach: Option<String>, message: String },  // NEW
       Serve { port: u16 },     // Shared
   }
   ```

3. **Test new commands thoroughly**
   - Manual tests for all scenarios
   - Integration tests for connection utility
   - Error path testing

4. **Cut over to new structure**
   - Remove deprecated `Chat` and `Attach` variants
   - Remove `chat.rs` and `attach.rs`
   - Update help text and documentation

### Rollback

If issues arise:
1. Revert `main.rs` to use `Chat` and `Attach` only
2. Keep new modules (they don't hurt)
3. Re-enable new subcommands as experimental features
4. Address feedback in follow-up change

## Open Questions

### Q1: Should `msg` support multiple messages or just one?

**Current decision**: One message only, then exit.

**Rationale**: Simple, explicit, matches name "one-shot".

**Alternative**: Support multiple messages like `sisyphus msg "hello" "world"`
**Cons**: Confusing with REPL, unclear where response separation happens
**Status**: Rejected. If multi-message needed, use REPL.

### Q2: Should `msg` have a timeout?

**Current decision**: No timeout (wait indefinitely for response).

**Rationale**: Matches current REPL behavior, simple implementation.

**Alternative**: Configurable timeout with exit code 2
**Pros**: Prevents hangs in automation
**Cons**: Adds complexity, may timeout legitimate long-running responses
**Status**: Defer to future enhancement if needed.

### Q3: Should TUI error if `--config` is provided with `--attach`?

**Current decision**: Yes, show error. `--config` only applies to local server.

**Rationale**: Clear semantics. Config shouldn't affect remote server.

**Alternative**: Ignore `--config` when `--attach` is provided
**Pros**: Less strict
**Cons**: Confusing, might mask user error
**Status**: Explicit error is clearer.

## Implementation Notes

### Testing Strategy

**Unit tests**:
- `connection.rs`: Test all paths (local spawn, remote connect, errors)
- Validate URL parsing, server ownership flag, error handling

**Integration tests**:
- Each command (repl/tui/msg) with local and remote modes
- Error paths (malformed URL, unreachable server, empty message)
- Server lifecycle (spawn → use → stop, connect → use → leave running)

**Manual tests** (tasks.md 9.1-9.10):
- Full workflow validation for each command
- Error message quality checks
- Cross-platform verification (Linux/macOS/Windows)

### Performance Considerations

- Server startup overhead: Only for local mode, one-time cost
- Connection pooling: Not implemented (simple, one-connection-per-command)
- Streaming response: Existing SSE mechanism reused (no change)

### Dependencies

- **New**: None (reuses existing `ServerManager`, `Client`, `clap`)
- **Modified**: `clap` enum in main.rs (breaking API change)
- **Removed**: None (internal refactor only)

## References

- Proposal: `openspec/changes/refactor-cli-command-structure/proposal.md`
- Spec delta: `openspec/changes/refactor-cli-command-structure/specs/cli-architecture/spec.md`
- Current CLI: `crates/cli/src/main.rs`
- Current commands: `crates/cli-core/src/commands/`
- ServerManager: `crates/cli-core/src/server_manager.rs`
