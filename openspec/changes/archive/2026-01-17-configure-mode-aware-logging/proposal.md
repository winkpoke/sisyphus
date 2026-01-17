# Change: Configure mode-aware logging levels

## Why
Server logs were always displayed in interactive CLI modes (REPL, TUI, msg) which cluttered user output. Users should have a clean, distraction-free experience in interactive modes while maintaining full logging visibility for server debugging.

## What Changes
- Add `init_with_defaults(default_level)` to logging system to support configurable defaults per mode
- Set server mode logging to INFO level (full visibility)
- Set REPL/TUI/msg modes logging to ERROR level (minimal interference)
- Suppress background server subprocess stderr in interactive modes (pipe to null)
- Users can still override with `RUST_LOG` environment variable

## Impact
- Affected specs: common-infra, cli-architecture
- Affected code:
  - `crates/common/src/logging.rs` - Added configurable logging initialization
  - `crates/cli/src/main.rs` - Mode-based logging configuration
  - `crates/cli-core/src/server_manager.rs` - Subprocess stderr handling
