# Tasks: Improve Test Coverage

## Overview
66 hours over 3 weeks to achieve 80% overall coverage with 90%+ for critical modules.

## Phase 1: Infrastructure Setup (Week 1)

- [ ] **Task 1.1**: Add coverage tracking to workspace
  - [ ] Add `cargo-tarpaulin` to workspace metadata in `Cargo.toml`
  - [ ] Create `scripts/coverage.sh` script with tarpaulin command
  - [ ] Add `coverage/` directory to `.gitignore`
  - [ ] Verify `cargo tarpaulin --workspace` runs successfully
  - [ ] Generate initial coverage report (baseline)
  - **Validation**: Run coverage script, confirm HTML report generated

- [ ] **Task 1.2**: Set up GitHub Actions CI/CD
  - [ ] Create `.github/workflows/` directory
  - [ ] Create `test.yml` workflow with jobs: test, lint, fmt-check
  - [ ] Add coverage job that runs tarpaulin and uploads to Codecov
  - [ ] Add matrix builds for Rust stable and beta versions
  - [ ] Add caching for Cargo dependencies
  - [ ] Configure Codecov token in repository secrets
  - [ ] Test workflow manually by pushing to test branch
  - **Validation**: All CI jobs pass on test push, coverage uploaded

- [ ] **Task 1.3**: Add development dependencies
   - [ ] Add `mockall = "0.12"` to workspace dev-dependencies
   - [ ] Add `wiremock = "0.6"` to `provider/Cargo.toml` dev-dependencies
   - [ ] Add `insta = "1.34"` to `core/Cargo.toml` dev-dependencies
   - [ ] Add `insta = "1.34"` to `provider/Cargo.toml` dev-dependencies
   - [ ] Add `tokio-test = "0.4"` to `core/Cargo.toml` dev-dependencies
   - [ ] Verify `tempfile` already exists in `crates/core/Cargo.toml` and `crates/tools/Cargo.toml`
   - [ ] Verify all dependencies compile with `cargo check --workspace`
   - [ ] Run `cargo tree --dev` to check for dependency conflicts
   - **Validation**: Cargo check passes, no dependency conflicts, tempfile confirmed present

- [ ] **Task 1.4**: Create test documentation and organization
   - [ ] Add "Testing" section to README.md with coverage command
   - [ ] Document how to run coverage locally
   - [ ] Document CI/CD workflow in CONTRIBUTING.md
   - [ ] Add badge for coverage to README.md
   - [ ] Create test data organization structure:
     - `crates/*/tests/fixtures/` for reusable test inputs
     - `crates/*/tests/golden/` for expected output files
     - (snapshots managed automatically by insta)
   - [ ] Document test data management strategy in AGENTS.md
   - **Validation**: Documentation is clear and accurate, test directories created

## Phase 2: Critical Security Tests (Week 1-2)

- [ ] **Task 2.1**: Test `tools/src/fs.rs`
  - [ ] Create `#[cfg(test)]` module in `fs.rs`
  - [ ] Add tests for file creation within sandbox
  - [ ] Add tests for file reading within sandbox
  - [ ] Add tests for file writing (overwrite)
  - [ ] Add tests for file deletion
  - [ ] Add tests for directory listing
  - [ ] Add tests for path traversal prevention (`../`)
  - [ ] Add tests for absolute path rejection
  - [ ] Add tests for symlink escape prevention
  - [ ] Add tests for permission errors (read-only file)
  - [ ] Add tests for large file handling
  - [ ] Verify 90%+ coverage for `fs.rs`
  - **Validation**: All tests pass, coverage target met

- [ ] **Task 2.2**: Test `tools/src/cmd.rs`
  - [ ] Create `#[cfg(test)]` module in `cmd.rs`
  - [ ] Add tests for safe command execution (echo, ls)
  - [ ] Add tests for stdout/stderr capture
  - [ ] Add tests for exit code handling
  - [ ] Add tests for dangerous command blocking (rm, sudo)
  - [ ] Add tests for shell injection prevention (;, &&, |, `)
  - [ ] Add tests for command argument escaping
  - [ ] Add tests for timeout handling
  - [ ] Add tests for environment variable injection prevention
  - [ ] Verify 90%+ coverage for `cmd.rs`
  - **Validation**: All tests pass, security edge cases covered

- [ ] **Task 2.3**: Test `core/src/agent.rs`
   - [ ] Create `tests/agent_loop_test.rs` integration test file
   - [ ] Add test for single-turn chat (no tools)
   - [ ] Add test for multi-turn conversation
   - [ ] Add test for tool execution flow (LLM → tool → result → LLM)
   - [ ] Add test for sequential tool calls in single message
   - [ ] Add test for provider error handling and retry
   - [ ] Add test for tool execution failure handling
   - [ ] Add test for max iterations limit enforcement
   - [ ] Add test for context window management
   - [ ] Add test for session history updates
   - [ ] Add test for concurrent chat requests (specify expected error behavior)
   - [ ] Add test for empty user message handling
   - [ ] Verify 90%+ coverage for `agent.rs` (aligned with spec)
   - **Validation**: All tests pass, agent loop verified

- [ ] **Task 2.4**: Test `core/src/session.rs` and `core/src/session/context.rs`
   - [ ] Create `tests/session_test.rs` integration test file
   - [ ] Add test for new session initialization
   - [ ] Add test for session status transitions (idle ↔ running)
   - [ ] Add test for message history append
   - [ ] Add test for history limit enforcement
   - [ ] Add test for context compaction trigger
   - [ ] Add test for session save to disk
   - [ ] Add test for session load from disk
   - [ ] Add test for session ID uniqueness
   - [ ] Add test for session metadata management
   - [ ] Add test for error state recovery
   - [ ] Add unit tests for `core/src/session/context.rs` (context compaction algorithm)
   - [ ] Verify 90%+ coverage for `session.rs` (aligned with spec)
   - [ ] Verify 80%+ coverage for `context.rs` (aligned with spec)
   - **Validation**: All tests pass, session lifecycle verified, context compaction tested

## Phase 3: Provider & Integration Tests (Week 2)

- [ ] **Task 3.1**: Test `provider/src/openai.rs`
   - [ ] Create `#[cfg(test)]` module in `openai.rs`
   - [ ] Add test for completion request serialization
   - [ ] Add test for stream request formatting
   - [ ] Add test for function call JSON generation
   - [ ] Add test for text response parsing
   - [ ] Add test for tool call response parsing
   - [ ] Add test for SSE chunk parsing
   - [ ] Add test for network error handling (timeout)
   - [ ] Add test for API error 429 (rate limit)
   - [ ] Add test for API error 500 (server error)
   - [ ] Add test for invalid JSON response handling
   - [ ] Add test for authentication error handling
   - [ ] Verify 80%+ coverage for `openai.rs` (aligned with spec)
   - **Validation**: All tests pass, request/response formats verified

- [ ] **Task 3.2**: Test provider traits and mock
  - [ ] Add tests for `LLMProvider` trait implementation
  - [ ] Add tests for `mock.rs` provider
  - [ ] Add test for mock provider response scripting
  - [ ] Add test for mock provider error injection
  - [ ] Add test for mock provider streaming (stub)
  - [ ] Verify all providers implement `LLMProvider` trait
  - **Validation**: Trait implementations verified, mocks functional

- [ ] **Task 3.3**: Test `server/src/lib.rs`
   - [ ] Create unit tests in `server/src/lib.rs`
   - [ ] Add test for `POST /api/v1/sessions` (create)
   - [ ] Add test for `GET /api/v1/sessions/:id` (retrieve)
   - [ ] Add test for `POST /api/v1/sessions/:id/chat` (chat)
   - [ ] Add test for `POST /api/v1/sessions/:id/approvals/:call_id` (approve)
   - [ ] Add test for `GET /api/v1/agents` (discovery)
   - [ ] Add test for CORS headers
   - [ ] Add test for error response formatting
   - [ ] Add test for request validation (missing fields)
   - [ ] Add test for authentication middleware (if applicable)
   - [ ] Add test for WebSocket upgrade handling (if applicable)
   - [ ] Add integration test for full request lifecycle
   - [ ] Verify 80%+ coverage for `server/src/lib.rs` (aligned with spec)
   - **Validation**: All tests pass, API contract verified

## Phase 4: UI & Client Tests (Week 3)

- [ ] **Task 4.1**: Test `cli-core/src/`
   - [ ] Add tests to `commands/repl.rs` for REPL execution
   - [ ] Add tests to `commands/msg.rs` for one-shot message
   - [ ] Add tests to `commands/serve.rs` for server lifecycle
   - [ ] Add tests to `ui/repl.rs` for REPL state
   - [ ] Add tests to `bootstrap.rs` for initialization
   - [ ] Add tests to `server_manager.rs` for server process management
   - [ ] Verify 70%+ coverage for `cli-core` (aligned with spec)
   - **Validation**: All tests pass, CLI behavior verified

- [ ] **Task 4.2**: Test `client/src/client.rs`
   - [ ] Create `#[cfg(test)]` module in `client.rs`
   - [ ] Add test for client initialization
   - [ ] Add test for chat request (HTTP client mock)
   - [ ] Add test for approval request
   - [ ] Add test for SSE event handling (streaming)
   - [ ] Add test for network error handling
   - [ ] Add test for connection pooling
   - [ ] Verify 70%+ coverage for `client.rs` (aligned with spec)
   - **Validation**: All tests pass, client API verified

- [ ] **Task 4.3**: Test TUI logic (focus on state, not rendering)
  - [ ] Add tests to `tui/src/tui/update.rs` (state transitions)
  - [ ] Add tests to `tui/src/tui/state.rs` (state management)
  - [ ] Add tests to `tui/src/tui/event.rs` (event handling)
  - [ ] Add tests to `tui/src/tui/action.rs` (action processing)
  - [ ] Add tests to `tui/src/tui/transcript.rs` (data formatting)
  - [ ] Add test for keyboard input handling
  - [ ] Add test for permission prompt display logic
  - [ ] Verify 60%+ coverage for TUI (logic only)
  - **Validation**: All tests pass, TUI state machine verified

## Phase 5: Quality Improvements (Ongoing)

- [ ] **Task 5.1**: Migrate to `mockall` framework
   - [ ] Document existing custom mocks in `crates/core/tests/`:
     - `MockProvider` in `agent_test.rs` (lines 14-49)
     - `MockTool` in `guardrails_test.rs` (lines 14-30)
     - `MockProvider` in `repro_deny_test.rs`
   - [ ] Add `mockall` dependency to `core/Cargo.toml` dev-dependencies
   - [ ] Replace `MockProvider` in `agent_test.rs` with `#[automock]`
   - [ ] Replace `MockProvider` in `guardrails_test.rs` with `#[automock]`
   - [ ] Replace `MockProvider` in `repro_deny_test.rs` with `#[automock]`
   - [ ] Replace `MockTool` implementations with `#[automock]`
   - [ ] Run all affected tests to verify migration works
   - [ ] Remove custom mock struct implementations
   - **Validation**: Mockall used consistently, no custom mocks, all tests pass

- [ ] **Task 5.2**: Add snapshot tests
   - [ ] Create snapshot for system prompt generation (`agent/prompt.rs`)
   - [ ] Create snapshot for tool schema generation
   - [ ] Create snapshot for OpenAI request formatting
   - [ ] Add test for prompt with custom template
   - [ ] Add test for prompt with environment variables
   - [ ] Run `cargo insta review` to verify snapshots
   - [ ] Create `.insta/review.sh` script
   - **Validation**: Snapshots pass, review workflow functional
   - **Note**: `insta` dependency already added in Task 1.3

- [ ] **Task 5.3**: Add property-based tests (if time permits)
   - [ ] Document invariants to test for context compaction (e.g., always reduces tokens)
   - [ ] Document invariants to test for message serialization (e.g., round-trip equality)
   - [ ] Document invariants to test for tool argument validation (e.g., schema compliance)
   - [ ] Add `proptest = "1.4"` dependency to `core/Cargo.toml`
   - [ ] Add property test for context compaction (always reduces tokens)
   - [ ] Add property test for message serialization round-trip
   - [ ] Add property test for tool argument validation
   - [ ] Configure property test iterations (100-1000)
   - [ ] Verify properties hold across random inputs
   - **Validation**: Property tests pass, invariants verified
   - **Note**: This is optional if core deadlines are at risk

- [ ] **Task 5.4**: Add benchmark tests
  - [ ] Add `criterion` dependency to workspace
  - [ ] Create benchmark for context compaction algorithm
  - [ ] Create benchmark for tool execution overhead
  - [ ] Create benchmark for message serialization
  - [ ] Add `benches/` directory structure
  - [ ] Run benchmarks and establish baseline
  - **Validation**: Benchmarks run, performance baseline established

## Phase 6: Validation & Documentation

- [ ] **Task 6.1**: Verify overall coverage targets
  - [ ] Run `cargo tarpaulin --workspace --out Html`
  - [ ] Check overall coverage is 80%+
  - [ ] Check critical modules are 90%+:
    - `tools/src/fs.rs`
    - `tools/src/cmd.rs`
    - `core/src/agent.rs`
    - `core/src/session.rs`
  - [ ] Check high-priority modules are 80%+:
    - `provider/src/openai.rs`
    - `server/src/lib.rs`
    - `core/src/session/context.rs`
  - [ ] Check medium-priority modules are 70%+:
    - `cli-core/`
    - `common/src/`
    - `client/`
  - [ ] Generate coverage badge for README
  - **Validation**: All coverage targets met, badge displays correctly

- [ ] **Task 6.2**: Performance validation
   - [ ] Measure baseline test execution time with `cargo test --workspace --timings`
   - [ ] Run `cargo test --workspace -- --ignored` to verify no tests are ignored unintentionally
   - [ ] Measure total test execution time
   - [ ] Optimize slow tests (>1s) if any
   - [ ] Verify test execution time is <30 seconds total (after optimization)
   - [ ] Add `#[ignore]` attributes to intentionally slow tests
   - [ ] Document baseline and optimized times for comparison
   - **Validation**: Tests are fast enough for CI, no unexpected ignores, baseline measured

- [ ] **Task 6.3**: Final documentation updates
   - [ ] Update TEST_STRATEGY.md with actual tooling used
   - [ ] Document test patterns in AGENTS.md
   - [ ] Add testing checklist to PR template
   - [ ] Update README.md with coverage badge
   - [ ] Add "How to Run Tests Locally" section
   - [ ] Document snapshot review workflow
   - [ ] Document test data organization strategy
   - **Validation**: Documentation is comprehensive and accurate

- [ ] **Task 6.4**: Optional - Mock-Real Behavior Drift Detection
   - [ ] Add `live-test` feature flag to relevant crates
   - [ ] Create contract test comparing mock responses to real provider responses
   - [ ] Document which mocks drift from real behavior
   - [ ] Update mocks to match real behavior or document intentional differences
   - **Validation**: Contract tests pass (when run with feature flag), mock behavior documented
   - **Note**: This is optional and should only be done if time permits

## Dependencies & Parallelization

**Dependencies (must complete in order):**
1. Phase 1 (Infrastructure) must be completed first
2. Phase 2 (Critical Security) depends on infrastructure for coverage verification
3. Phase 3 (Provider/Integration) can parallelize with Phase 2 after infrastructure
4. Phase 4 (UI/Client) can run independently after Phase 1
5. Phase 5 (Quality) can run in parallel with all phases
6. Phase 6 (Validation) must complete all other phases first

**Parallelizable Tasks:**
- Phase 2 tasks can run in parallel (different developers)
- Phase 3 tasks can run in parallel with Phase 2
- Phase 4 tasks can run in parallel with Phase 2
- Phase 5 can run incrementally alongside other phases
