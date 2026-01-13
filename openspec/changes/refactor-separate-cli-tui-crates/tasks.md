## Implementation Tasks

### 1. Setup and Preparation
- [ ] 1.1 Create `crates/tui` directory structure with `src/` and `Cargo.toml`
- [ ] 1.2 Initialize `crates/tui/src/lib.rs` with empty module structure
- [ ] 1.3 Update workspace `Cargo.toml` to include `tui` in members list
- [ ] 1.4 Add optional `tui` feature to `crates/cli/Cargo.toml` with `dep:tui`
- [ ] 1.5 Run `cargo check --workspace` to verify workspace compiles

### 2. TUI Crate Skeleton
- [ ] 2.1 Create `crates/tui/Cargo.toml` with TUI-specific dependencies (ratatui, crossterm, arboard)
- [ ] 2.2 Add CLI crate as dependency to `crates/tui/Cargo.toml`
- [ ] 2.3 Verify `cargo check -p tui` compiles successfully
- [ ] 2.4 Verify `cargo build --bin sisyphus` (without tui feature) still compiles

### 3. Move TUI Code
- [ ] 3.1 Move `crates/cli/src/ui/tui/` directory to `crates/tui/src/`
- [ ] 3.2 Create `crates/tui/src/lib.rs` and re-export TUI public interface
- [ ] 3.3 Update internal imports in TUI modules (remove `crate::` prefixes for CLI items)
- [ ] 3.4 Verify `cargo check -p tui` identifies missing dependencies
- [ ] 3.5 Fix all compilation errors in TUI crate

### 4. Update CLI to Use Optional TUI
- [ ] 4.1 Remove TUI-specific dependencies from `crates/cli/Cargo.toml` main section
- [ ] 4.2 Conditionally include TUI dependencies in `crates/cli/Cargo.toml` under `[features.tui]`
- [ ] 4.3 Update `crates/cli/src/commands/chat.rs` to conditionally import TUI based on feature
- [ ] 4.4 Update `crates/cli/src/main.rs` to conditionally enable `--tui` flag
- [ ] 4.5 Verify `cargo build --bin sisyphus` (CLI-only) compiles without errors

### 5. TUI Feature Integration
- [ ] 5.1 Verify `cargo build --bin sisyphus --features tui` compiles successfully
- [ ] 5.2 Test that `sisyphus chat --tui` works with TUI feature enabled
- [ ] 5.3 Test that `sisyphus chat` (without --tui) uses REPL
- [ ] 5.4 Verify binary size difference: CLI-only vs CLI+TUI builds

### 6. Shared Utilities Validation
- [ ] 6.1 Verify `banner.rs` and `completer.rs` remain in `crates/cli/src/ui/`
- [ ] 6.2 Confirm TUI crate can access shared utilities via CLI dependency
- [ ] 6.3 Test REPL uses banner and completer correctly
- [ ] 6.4 Test TUI uses banner and completer correctly (if applicable)

### 7. Documentation Updates
- [ ] 7.1 Update `README.md` to document TUI feature flag usage
- [ ] 7.2 Update architecture section to reflect new crate structure
- [ ] 7.3 Update build instructions with TUI feature examples
- [ ] 7.4 Verify all documentation changes are accurate

### 8. Testing and Validation
- [ ] 8.1 Run `cargo test --workspace` to ensure all tests pass
- [ ] 8.2 Run `cargo clippy --workspace` to check for warnings
- [ ] 8.3 Run `cargo fmt --all` to ensure code formatting
- [ ] 8.4 Manual test: Start REPL-only session and verify functionality
- [ ] 8.5 Manual test: Start TUI session with `--tui` flag and verify functionality
- [ ] 8.6 Manual test: Verify `sisyphus serve` and `sisyphus attach` work correctly

### 9. Cleanup and Finalization
- [ ] 9.1 Remove any temporary files or dead code from migration
- [ ] 9.2 Verify git diff shows only intended changes
- [ ] 9.3 Run final `cargo build --workspace` to confirm all targets compile
- [ ] 9.4 Run `cargo build --workspace --all-features` to confirm feature integration
- [ ] 9.5 Document any known issues or limitations in design.md if discovered during implementation

## Validation Criteria

### Success Metrics
- CLI-only build succeeds without TUI dependencies
- TUI feature enables TUI functionality correctly
- All existing tests pass
- Binary size reduction for CLI-only builds
- No circular dependencies
- Shared utilities accessible to both modes

### Failure Rollback
If any task fails validation:
1. Document the failure and attempted fixes
2. Roll back to last working commit
3. Create new task to address the failure
4. Resume implementation after rollback point
