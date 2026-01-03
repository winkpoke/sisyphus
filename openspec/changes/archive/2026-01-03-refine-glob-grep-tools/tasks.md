## 1. Specification
- [x] 1.1 Update `tooling` spec deltas for regex dialect, columns, truncation, and binary handling
- [x] 1.2 Validate spec deltas reference existing requirements after archiving dependency

## 2. Implementation
- [x] 2.1 Implement UTF-8 character column computation for `grep` content results
- [x] 2.2 Implement deterministic binary/non-UTF-8 handling for `grep`
- [x] 2.3 Fail fast on invalid override patterns for `glob` and `grep`
- [x] 2.4 Implement deterministic truncation rules for `grep` content output
- [x] 2.5 Refactor shared sandbox and denylist validation into a common helper

## 3. Tests
- [x] 3.1 Add unit tests for UTF-8 column computation
- [x] 3.2 Add unit tests for binary file skipping and invalid UTF-8 handling
- [x] 3.3 Add unit tests for invalid override pattern errors
- [x] 3.4 Add unit tests for content truncation containing the match
- [x] 3.5 Add unit tests for invalid regex errors

## 4. Verification
- [x] 4.1 Run `cargo test --workspace`
- [x] 4.2 Run `cargo clippy --workspace --all-targets --all-features`
