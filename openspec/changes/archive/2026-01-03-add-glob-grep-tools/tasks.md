## 1. Implementation
- [x] 1.1 Implement `glob` tool in `crates/tools`
- [x] 1.2 Implement `grep` tool in `crates/tools`
- [x] 1.3 Enforce ignore, include_ignored, and exclude semantics
- [x] 1.4 Enforce sandboxing, symlink escape prevention, and denylist
- [x] 1.5 Enforce output caps and deterministic truncation

## 2. Integration
- [x] 2.1 Register `glob` and `grep` during agent bootstrap
- [x] 2.2 Ensure tool schemas match the spec deltas

## 3. Tests
- [x] 3.1 Add unit tests for schema and path rejection
- [x] 3.2 Add unit tests for denylist and symlink escape prevention
- [x] 3.3 Add integration tests for ignore and selective include behavior
- [x] 3.4 Add integration tests for grep output modes and truncation

## 4. Verification
- [x] 4.1 Run `cargo test` for the workspace
- [x] 4.2 Run `cargo clippy --workspace --all-targets --all-features`
