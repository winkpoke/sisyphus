## Implementation Tasks

### 1. Verification and Preparation
- [ ] 1.1 Confirm `main.rs` imports all functionality from `sisyphus_cli_lib`
- [ ] 1.2 Verify `crates/cli-lib` contains all referenced code (repl.rs, commands/, server_manager.rs, bootstrap.rs)
- [ ] 1.3 Check that no external consumers import from `crates/cli` (should only import from cli-lib)
- [ ] 1.4 Run `cargo build --bin sisyphus` to confirm current state compiles

### 2. Rename cli-lib to cli-core
- [ ] 2.1 Rename directory: `crates/cli-lib` → `crates/cli-core`
- [ ] 2.2 Update `crates/cli-core/Cargo.toml`: `sisyphus-cli-lib` → `sisyphus-cli-core`
- [ ] 2.3 Update workspace `Cargo.toml`: add `crates/cli-core` to members, remove `crates/cli-lib`
- [ ] 2.4 Update workspace `Cargo.toml` dependencies: `sisyphus_cli_lib` → `sisyphus_cli_core`
- [ ] 2.5 Update `crates/cli/Cargo.toml`: dependency path `../cli-lib` → `../cli-core`
- [ ] 2.6 Update `crates/cli/Cargo.toml`: dependency package `sisyphus-cli-lib` → `sisyphus-cli-core`
- [ ] 2.7 Update `crates/tui/Cargo.toml`: dependency path `../cli-lib` → `../cli-core`
- [ ] 2.8 Update `crates/tui/Cargo.toml`: dependency package `sisyphus-cli-lib` → `sisyphus-cli-core`
- [ ] 2.9 Update all code imports: `sisyphus_cli_lib` → `sisyphus_cli_core`
- [ ] 2.10 Verify `cargo check --workspace` compiles after rename

### 3. Remove Duplicate Source Files
- [ ] 2.1 Delete `crates/cli/src/bootstrap.rs`
- [ ] 2.2 Delete `crates/cli/src/server_manager.rs`
- [ ] 2.3 Delete `crates/cli/src/commands/` directory and all contents
- [ ] 2.4 Delete `crates/cli/src/ui/` directory and all contents

### 4. Verification After Changes
- [ ] 4.1 Run `cargo build --bin sisyphus` - should succeed with only `main.rs`
- [ ] 4.2 Run `cargo check --package sisyphus` - should pass without errors
- [ ] 4.3 Run `cargo test --workspace` - all tests should still pass
- [ ] 4.4 Verify `main.rs` still compiles and imports work correctly
- [ ] 4.5 Verify renamed package `sisyphus-cli-core` compiles correctly

### 5. Feature Validation
- [ ] 5.1 Test `sisyphus chat` (REPL mode) works correctly
- [ ] 5.2 Test `sisyphus serve` starts server correctly
- [ ] 5.3 Test `sisyphus attach http://localhost:PORT` connects correctly
- [ ] 5.4 Test `sisyphus chat --tui` works (if tui feature available)

### 6. Documentation Updates
- [ ] 6.1 Update any references to removed files in documentation
- [ ] 6.2 Update any references to `cli-lib` package name in documentation
- [ ] 6.3 Verify README.md still reflects correct architecture
- [ ] 6.4 Confirm code comments don't reference removed modules or old package name

### 7. Final Validation
- [ ] 7.1 Run `cargo fmt --all` to ensure formatting
- [ ] 7.2 Run `cargo clippy --workspace` to check for warnings
- [ ] 7.3 Verify git diff shows directory rename and file deletions
- [ ] 7.4 Verify package name change from `sisyphus-cli-lib` to `sisyphus-cli-core`
- [ ] 7.5 Final build test: `cargo build --workspace --all-features`

## Validation Criteria

### Success Metrics
- Binary builds successfully with only `main.rs` in `crates/cli/src/`
- Package renamed from `sisyphus-cli-lib` to `sisyphus-cli-core`
- Directory renamed from `cli-lib` to `cli-core`
- All existing functionality (chat, serve, attach) works identically
- No compilation errors or warnings
- All tests pass
- Source tree is cleaner with clear separation: binary (cli) vs. library (cli-core)
- All workspace dependencies correctly reference new package name

### Failure Rollback
If any task fails validation:
1. Document the failure and symptom
2. Restore deleted files from git
3. Investigate root cause before retrying removal
4. Run `openspec validate cleanup-cli-binary-separation --strict` to ensure correctness
