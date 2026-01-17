# Change: Refactor CLI Command Structure

## Why

The current CLI command structure (`chat`, `serve`, `attach`) mixes local and remote concerns in a way that's not intuitive. Users must remember three different commands for what are essentially two operations (interactive vs one-shot) with optional remote connection. The proposed structure (`repl`, `tui`, `msg`, `serve`) with an optional `--attach` flag makes the distinction clearer: choose your interface (REPL/TUI/message), then optionally connect to remote. This also aligns with user expectations where "repl" and "tui" are explicit interface choices, and "serve" is a standalone command for dedicated server mode.

## What Changes

- **BREAKING**: Rename and restructure CLI commands:
  - Remove: `chat` subcommand with `--tui` flag
  - Remove: `attach` subcommand
  - Add: `repl` subcommand (local REPL by default, supports `--attach <url>`)
  - Add: `tui` subcommand (local TUI by default, supports `--attach <url>`)
  - Add: `msg` subcommand (one-shot messages, supports `--attach <url>`)
  - Keep: `serve` subcommand (unchanged, for dedicated server mode)
- New command structure:
  ```bash
  sisyphus repl                    # local REPL
  sisyphus tui                     # local TUI
  sisyphus msg "message..."        # one-shot local message
  sisyphus serve --port 3000       # dedicated server (unchanged)
  sisyphus repl --attach <url>     # remote REPL
  sisyphus tui --attach <url>      # remote TUI
  sisyphus msg --attach <url> "..." # remote one-shot message
  ```
- Update clap argument parsing in `crates/cli/src/main.rs`
- Refactor `crates/cli-core/src/commands/` to match new structure:
  - Create `repl.rs` module
  - Create `tui.rs` module
  - Create `msg.rs` module
  - Keep `serve.rs` module (unchanged)
  - Remove or refactor `chat.rs` (functionality split into repl/tui/msg)

## Impact

- **Affected specs**:
  - `cli-architecture` (MODIFIED - update command structure, add one-shot messaging, update remote connection pattern)
- **Affected code**:
  - `crates/cli/src/main.rs` - updated clap `Subcommand` enum
  - `crates/cli-core/src/commands/chat.rs` - split into repl/tui/msg modules
  - `crates/cli-core/src/commands/attach.rs` - functionality moved to repl/tui/msg via `--attach` flag
  - `crates/cli-core/src/commands/serve.rs` - unchanged
  - `crates/cli-core/src/commands/connection.rs` - NEW shared utility for connection setup
  - `crates/cli-core/src/lib.rs` - update exports
  - User documentation and help text - updated to reflect new commands

## Migration Guide

### Breaking Changes

| Old Command | New Command | Notes |
|------------|-------------|-------|
| `sisyphus chat` | `sisyphus repl` | REPL interface now explicit |
| `sisyphus chat --tui` | `sisyphus tui` | TUI interface now explicit |
| `sisyphus attach <url>` | `sisyphus repl --attach <url>` | Remote connection via flag |
| N/A | `sisyphus msg "message"` | New one-shot messaging capability |

### Migration Steps

1. **Update scripts and aliases**: Replace old command patterns with new equivalents
2. **Update documentation**: Replace `chat` with `repl` or `tui` as appropriate
3. **Update `attach` usage**: Change to use `--attach <url>` flag with `repl`/`tui`/`msg`

### Compatibility

This is a **hard breaking change** with no backward compatibility period. Existing workflows using `chat` or `attach` will fail immediately.

### Rationale

The new structure provides clearer separation of concerns:
- **Interface choice first** (repl/tui/msg)
- **Connection mode second** (local via default, remote via --attach)
- **Explicit semantics** (no ambiguity about `--tui` flag on `chat`)
