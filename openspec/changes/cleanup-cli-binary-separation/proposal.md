# Change: Clean Up CLI Binary and Remove Duplicate Code

## Why

The `crates/cli` crate contains duplicate code that mirrors `crates/cli-lib` implementation, creating confusion and maintenance burden. Files like `bootstrap.rs`, `server_manager.rs`, `commands/`, and `ui/` (except for `main.rs`) exist in `crates/cli/src/` but are not actually used—the code in `main.rs` imports from `sisyphus_cli_lib` instead. This violates the clean separation principle where the binary crate should be a thin entry point and actual CLI/REPL logic should live in a shared library.

## What Changes

- Remove all unused duplicate files from `crates/cli/src/`:
  - Delete `bootstrap.rs`
  - Delete `server_manager.rs`
  - Delete `commands/` directory
  - Delete `ui/` directory
- Keep only `main.rs` in `crates/cli/src/` as binary entry point
- Rename `crates/cli-lib` to `crates/cli-core` for clearer semantics:
  - Rename directory: `cli-lib` → `cli-core`
  - Rename package: `sisyphus-cli-lib` → `sisyphus-cli-core`
  - Update workspace members list
  - Update all dependent crates (`cli`, `tui`)
- Ensure `crates/cli-core` continues to contain all CLI/REPL implementation logic
- **BREAKING**: Package name change from `sisyphus-cli-lib` to `sisyphus-cli-core`

## Impact

- Affected specs:
  - `cli-architecture` (MODIFIED - clarify binary crate is thin entry point only, update crate naming)
- Affected code:
  - `crates/cli/src/` - reduced from 8 files to 1 file (`main.rs` only)
  - `crates/cli-lib/` → `crates/cli-core/` - renamed for clearer semantics
  - `crates/cli/Cargo.toml` - updated dependency reference
  - `crates/tui/Cargo.toml` - updated dependency reference
  - `Cargo.toml` (workspace) - updated members list and dependencies
  - Binary build output - identical functionality, improved semantic clarity
