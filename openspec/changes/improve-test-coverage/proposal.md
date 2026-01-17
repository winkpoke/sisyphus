# Change: Improve Test Coverage

## Why

Current test coverage is 21.7% (77 tests across 69 source files), with critical security and core logic modules untested. This poses risks to code quality, security, and maintainability. The TEST_STRATEGY.md document outlines comprehensive testing requirements (mockall, wiremock, insta) but these tools are not configured in the project. Additionally, there is no CI/CD pipeline to ensure tests run on every commit and no coverage tracking mechanism.

## What Changes

**Phase 1: Infrastructure**
- Add `cargo-tarpaulin` to workspace for coverage tracking
- Set up GitHub Actions CI/CD pipeline with automated testing
- Add coverage script and report generation
- Configure Codecov integration for coverage reporting

**Phase 2: Critical Security Tests**
- Add comprehensive tests for `tools/src/fs.rs` (file operations, sandbox enforcement)
- Add comprehensive tests for `tools/src/cmd.rs` (shell execution, injection prevention)
- Add unit tests for `core/src/agent.rs` (agent loop, tool execution flow)
- Add unit tests for `core/src/session.rs` (session lifecycle, state management)

**Phase 3: Provider & Integration Tests**
- Add unit tests for `provider/src/openai.rs` using wiremock
- Add tests for provider trait implementations
- Add unit tests for `server/src/lib.rs` (HTTP handlers, middleware)

**Phase 4: UI & Client Tests**
- Add tests for CLI core components (REPL, commands)
- Add tests for client library
- Add tests for TUI state management and logic

**Phase 5: Quality Improvements**
- Migrate to `mockall` for consistent mocking
- Add snapshot testing using `insta` for prompts and JSON schemas
- Add property-based tests for critical algorithms

## Impact

**Affected Specs:**
- `workspace/spec.md` - CI/CD, coverage tracking requirements
- `agent-core/spec.md` - Agent loop testing requirements
- `session-core/spec.md` - Session management testing requirements
- `tooling/spec.md` - Tool security testing requirements
- `llm-provider/spec.md` - Provider testing requirements
- `server-core/spec.md` - Server handler testing requirements
- `cli-architecture/spec.md` - CLI testing requirements

**Affected Code:**
- `Cargo.toml` (workspace) - Add dev-dependencies: tarpaulin, mockall, insta, wiremock
- `.github/workflows/` - Add CI/CD pipeline
- `crates/*/Cargo.toml` - Add dev-dependencies where needed
- `crates/core/src/agent.rs` - Add unit tests
- `crates/core/src/session.rs` - Add unit tests
- `crates/tools/src/fs.rs` - Add comprehensive tests
- `crates/tools/src/cmd.rs` - Add comprehensive tests
- `crates/provider/src/openai.rs` - Add unit tests with wiremock
- `crates/server/src/lib.rs` - Add unit tests
- `crates/cli-core/` - Add tests for commands and REPL
- `crates/client/` - Add tests for client library

**Dependencies Added:**
- `cargo-tarpaulin` (workspace) - Coverage tracking
- `mockall` (dev) - Mocking framework
- `wiremock` (dev) - HTTP mocking for provider tests
- `insta` (dev) - Snapshot testing
- `tokio-test` (dev) - Async testing utilities

**Dependencies Already Present (Verified):**
- `tempfile` (dev) - Already in `crates/core` and `crates/tools`

**Estimated Effort:**
- 66 hours over 3 weeks
- Target: 80% overall coverage, 90%+ for critical modules

**Known Issues Addressed:**
- 14 critical bugs fixed (dependency duplicates, contradictory targets, missing modules)
- 10 design flaws addressed (security gaps, concurrency, unrealistic expectations)
