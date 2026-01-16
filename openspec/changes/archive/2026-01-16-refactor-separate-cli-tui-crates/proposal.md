# Change: Separate CLI and TUI into Distinct Crates

## Why

The current `crates/cli` crate contains both the headless REPL interface and the full-featured TUI, causing unnecessary dependency bloat and architectural coupling. This violates the project's stated design principle of being "headless by design" and makes it harder to test and evolve each UI mode independently.

## What Changes

- Create a new `crates/tui` crate for the Terminal User Interface
- Move TUI-specific code from `crates/cli/src/ui/tui/` to the new crate
- Remove TUI dependencies (`ratatui`, `crossterm`, `arboard`) from `crates/cli/Cargo.toml`
- Add TUI as an optional feature to `crates/cli` that depends on the new TUI crate
- Preserve shared utilities (banner, completer) in a common location
- **BREAKING**: Users building with `--all-features` will see an additional crate in the workspace

## Impact

- Affected specs:
  - `cli-architecture` (MODIFIED - updates to reflect new crate structure)
  - `cli-tui` (ADDED - new capability for standalone TUI crate)
- Affected code:
  - `crates/cli/` - becomes a thin binary with optional TUI integration
  - `crates/tui/` - new crate containing all TUI code
  - `Cargo.toml` - workspace members updated
  - Build artifacts - smaller CLI-only binaries, separate TUI dependency tree
