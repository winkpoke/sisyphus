# Design: Test Coverage Infrastructure

## Context

Sisyphus currently has 77 tests across 69 source files (21.7% coverage). Critical security-sensitive modules (`tools/src/fs.rs`, `tools/src/cmd.rs`) and core logic (`core/src/agent.rs`, `core/src/session.rs`) are untested. The TEST_STRATEGY.md documents testing requirements and recommended tooling (mockall, wiremock, insta, tempfile, tokio-test), but these tools are not configured in Cargo.toml files.

**Current State:**
- No CI/CD pipeline
- No coverage tracking (tarpaulin, grcov, codecov not configured)
- Custom mock implementations instead of standardized framework
- Missing snapshot testing for generated content
- Security-critical file and command execution tools untested

**Constraints:**
- Must follow headless architecture (tests should not depend on UI)
- Must use sandboxing for file system operations
- Must not call live LLM APIs in tests
- Must maintain fast test execution (< 30 seconds total)
- Tests must be deterministic

**Stakeholders:**
- Developers - Need fast feedback on code changes
- Security - File and shell tools must be secure
- CI/CD - Automated test execution and coverage reporting

## Goals / Non-Goals

### Goals
- Establish 80% overall test coverage with 90%+ for critical modules
- Set up CI/CD pipeline for automated testing on every push/PR
- Implement coverage tracking with Codecov integration
- Replace custom mocks with standardized `mockall` framework
- Add snapshot testing for prompts, JSON schemas, and generated content
- Ensure all security-sensitive code paths are tested

### Incremental Milestones
To ensure realistic progress tracking, coverage targets should be achieved in phases:
- **Week 1**: Baseline established (21.7% → 40% overall), infrastructure complete
- **Week 2**: Critical security modules tested (40% → 60% overall), providers tested
- **Week 3**: UI/client tests added (60% → 80% overall), quality improvements applied

If a milestone is missed, reassess scope and prioritize critical modules over lower-priority ones.

### Non-Goals
- 100% coverage (not all code is testable or worth testing)
- Integration tests that require live LLM providers
- UI rendering tests (focus on logic, not visual output)
- End-to-end browser-based tests (out of scope for this change)

## Decisions

### Decision 1: Use `cargo-tarpaulin` for Coverage Tracking

**What:**
Add `cargo-tarpaulin` to workspace dev-dependencies and configure it for coverage reporting.

**Why:**
- Native Rust integration, works with Cargo workspace
- Supports HTML and XML output formats
- Compatible with Codecov for CI/CD integration
- No complex build configuration needed

**Alternatives Considered:**
- `grcov` - Requires LLVM instrumentation, more complex setup
- Manual coverage counting - Not automated, error-prone
- `coverage.py` - Not native to Rust ecosystem

**Configuration:**
```toml
[workspace.metadata.tarpaulin]
# Output directory for coverage reports
out-dir = "target/tarpaulin"
# List of crates to include (default: all)
include = ["sisyphus_core", "tools", "provider"]
```

### Decision 2: GitHub Actions for CI/CD

**What:**
Create `.github/workflows/test.yml` with automated testing and coverage upload on every push and PR.

**Why:**
- GitHub Actions provides free CI/CD for public repositories
- Integrates seamlessly with Codecov for coverage visualization
- Matrix builds for Rust stable and beta versions
- Parallel test execution for fast feedback

**Workflow Structure:**
```yaml
on: [push, pull_request]
jobs:
  test:
    - cargo test --workspace
    - cargo clippy -- -D warnings
    - cargo fmt -- --check
  coverage:
    - cargo tarpaulin --workspace --out Xml
    - upload to codecov
```

### Decision 3: Mockall for Consistent Mocking

**What:**
Replace custom `MockProvider` and `MockTool` implementations with `mockall` framework.

**Why:**
- Standardized mocking reduces boilerplate
- Better IDE support and autocomplete
- Automatic trait implementation
- Support for expectations, sequences, and call counting

**Migration Pattern:**
```rust
// Before (custom mock)
struct MockProvider {
    responses: Arc<Mutex<Vec<Message>>>,
}

// After (mockall)
#[automock]
trait LLMProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<Message>;
}
```

**Rollout Strategy:**
- Phase 1: Add mockall to dev-dependencies
- Phase 2: Migrate critical tests (agent, session) first
- Phase 3: Migrate remaining tests incrementally
- Phase 4: Remove custom mock implementations

### Decision 4: Wiremock for Provider Tests

**What:**
Use `wiremock` to mock HTTP endpoints for testing `provider/src/openai.rs`.

**Why:**
- HTTP mocking is essential for testing provider implementations
- Avoids calling live OpenAI API
- Enables testing error scenarios (429, 500, network timeout)
- Supports request/response verification

**Test Pattern:**
```rust
#[tokio::test]
async fn test_openai_completion_success() {
    let mock_server = MockServer::start().await;
    mock_server.mock(|when, then| {
        when.method(POST).path("/v1/chat/completions")
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(expected_response));
    });

    let provider = OpenAIProvider::new_with_base_url(mock_server.uri());
    let result = provider.complete(request).await;
    assert!(result.is_ok());
}
```

### Decision 5: Tempfile for File System Tests

**What:**
Use `tempfile::tempdir()` to sandbox all file operations in `tools/src/fs.rs`.

**Why:**
- Prevents test pollution of developer's file system
- Ensures path traversal attacks are caught
- Automatic cleanup after test completion
- Isolates test execution

**Sandbox Pattern:**
```rust
#[tokio::test]
async fn test_file_operations_sandboxed() {
    let temp_dir = TempDir::new().unwrap();
    let fs_tool = FsTool::new(temp_dir.path());

    // All operations confined to temp_dir
    fs_tool.write_file("test.txt", "content").await.unwrap();

    // Verify file exists only in temp_dir
    assert!(temp_dir.path().join("test.txt").exists());
}
```

### Decision 6: Insta for Snapshot Testing

**What:**
Use `insta` for snapshot testing of generated prompts, JSON schemas, and tool outputs.

**Why:**
- Detects unintended changes in generated content
- Simplifies testing of complex data structures
- Easy review of snapshot diffs with `cargo insta review`
- Inline snapshots avoid separate files

**Snapshot Targets:**
- System prompt generation (`core/src/agent/prompt.rs`)
- Tool schemas (`provider/src/openai.rs`)
- Completion request formatting
- Error response messages

**Pattern:**
```rust
#[tokio::test]
fn test_system_prompt_generation() {
    let config = AgentConfig::default();
    let prompt = build_prompt(&config);
    insta::assert_snapshot!(prompt);
}
```

### Decision 7: Test Organization Strategy

**What:**
Organize tests into unit tests (#[cfg(test)] modules) and integration tests (tests/ directories) following Rust conventions.

**Why:**
- Unit tests are fast, run on every code change
- Integration tests verify component interactions
- Clear separation of concerns
- Parallel test execution optimization

**Guidelines:**
- Unit tests: Test single functions/modules, use mocks
- Integration tests: Test end-to-end flows, use real dependencies
- Security tests: Test edge cases, attack vectors
- Async tests: Use `#[tokio::test]` consistently

### Decision 8: Coverage Targets by Priority

**What:**
Define tiered coverage targets to prioritize critical code paths.

**Targets:**
- **P0 (Critical):** 90%+ coverage
  - `tools/src/fs.rs` - File operations, security
  - `tools/src/cmd.rs` - Shell execution, injection
  - `core/src/agent.rs` - Agent loop, tool execution
  - `core/src/session.rs` - Session state, permissions

- **P1 (High):** 80%+ coverage
  - `provider/src/openai.rs` - LLM integration
  - `server/src/lib.rs` - API handlers
  - `core/src/session/context.rs` - Context compaction

- **P2 (Medium):** 70%+ coverage
  - `cli-core/` - CLI commands, REPL
  - `common/src/` - Shared utilities
  - `client/` - Client library

- **P3 (Low):** 60%+ coverage
  - `tui/` - UI components (focus on logic, not rendering)

## Risks / Trade-offs

### Risk 1: Test Execution Time Increase

**Risk:**
Adding 66 hours of tests may slow down CI pipeline and developer workflow.

**Mitigation:**
- Use `#[ignore]` attribute for slow tests (>1s)
- Enable parallel test execution in CI (cargo test --jobs)
- Cache dependencies and build artifacts
- Use mockall for fast deterministic tests
- Limit property-based test iterations

### Risk 2: Flaky Tests with Async Code

**Risk:**
Async tests using Tokio may be flaky due to timing issues.

**Mitigation:**
- Use `tokio::test::time::pause()` for deterministic timing
- Avoid sleep() calls in tests
- Use futures::executor::block_on for deterministic execution
- Test timeout mechanisms with shorter limits

### Risk 3: Snapshot Drift

**Risk:**
Insta snapshots may drift over time, requiring frequent updates.

**Mitigation:**
- Use `cargo insta review` to review changes intentionally
- Keep snapshots in version control
- Document intentional snapshot changes in commit messages
- Separate snapshots from implementation where appropriate

### Risk 4: Mock-Real Behavior Mismatch

**Risk:**
Mock implementations may not perfectly match real provider/tool behavior.

**Mitigation:**
- Document mock behavior expectations clearly
- Update mocks when real implementations change
- Use integration tests to verify mock accuracy
- Add error response scenarios to mock behavior
- Add contract tests with real providers under `--feature live-test` flag (optional)

### Risk 5: Maintenance Overhead

**Risk:**
High test coverage increases maintenance burden when refactoring.

**Mitigation:**
- Keep tests focused and orthogonal
- Use test helpers to reduce duplication
- Use snapshot tests for complex outputs
- Document testing patterns in AGENTS.md
- Periodic test cleanup (remove obsolete tests)

## Implementation Dependencies

### Phase Sequence

1. **Infrastructure (Week 1):** Coverage + CI/CD
   - Must be done first to establish baseline
   - Enables continuous monitoring of progress

2. **Critical Security (Week 1-2):** Tools + Core
   - Highest risk if untested
   - Blocks on infrastructure for coverage tracking

3. **Provider/Integration (Week 2):** Provider + Server
   - Medium priority, moderate risk
   - Can parallelize with Critical Security phase

4. **UI/Client (Week 3):** CLI + TUI + Client
   - Lower priority, low risk
   - Independent of core logic

5. **Quality (Ongoing):** Mockall migration + Snapshots
   - Incremental, low-risk
   - Can be done alongside other phases

### Tooling Trade-offs

| Tool | Benefit | Cost |
|-------|---------|-------|
| `mockall` | Less boilerplate, better IDE support | Learning curve for team |
| `wiremock` | Comprehensive HTTP testing | Setup complexity for endpoints |
| `insta` | Easy regression detection | Review overhead for intentional changes |
| `tarpaulin` | Native coverage tracking | Slower than cargo test |

**Decision:** Benefits outweigh costs. These tools are standard in Rust ecosystem and will improve long-term maintainability.

### Decision 9: Test Data Management Strategy

**What:**
Establish organizational structure for test fixtures, golden test files, and test data.

**Why:**
- Prevents test data duplication
- Makes tests easier to understand and maintain
- Enables consistent test data across test files
- Supports golden testing for snapshots

**Organization:**
```
crates/
├── core/
│   ├── tests/
│   │   ├── fixtures/         # Test input data (JSON, YAML)
│   │   ├── snapshots/        # Insta snapshots (managed by cargo insta)
│   │   └── golden/          # Expected output files for golden tests
│   └── src/
```

**Guidelines:**
- Use `fixtures/` for reusable test inputs
- Let `insta` manage `snapshots/` directory
- Use `golden/` for file-based comparison tests
- Keep fixture files small and focused
- Document fixture purpose in file headers

**Mitigation:**
- Regular cleanup of obsolete fixtures
- Version golden files alongside code changes
- Use descriptive fixture file names
