# workspace Specification (Delta)

## ADDED Requirements

### Requirement: CI/CD Pipeline
The workspace SHALL provide automated CI/CD pipeline for testing, linting, and coverage tracking on every push and pull request.

#### Scenario: Tests run on every push
- **GIVEN** a repository with CI/CD configured
- **WHEN** code is pushed to `main` or any branch
- **THEN** GitHub Actions runs `cargo test --workspace`
- **AND** all tests must pass
- **AND** CI status is reported in PR/commit

#### Scenario: Clippy checks run
- **GIVEN** a repository with CI/CD configured
- **WHEN** code is pushed
- **THEN** GitHub Actions runs `cargo clippy --workspace -- -D warnings`
- **AND** any warnings must fail the CI job

#### Scenario: Format checks run
- **GIVEN** a repository with CI/CD configured
- **WHEN** code is pushed
- **THEN** GitHub Actions runs `cargo fmt -- --check`
- **AND** any formatting issues must fail the CI job

#### Scenario: Coverage reports generated
- **GIVEN** a repository with CI/CD configured
- **WHEN** CI runs in coverage job
- **THEN** GitHub Actions runs `cargo tarpaulin --workspace --out Xml`
- **AND** coverage report is generated successfully

#### Scenario: Coverage uploaded to Codecov
- **GIVEN** a repository with CI/CD configured
- **WHEN** CI runs coverage job
- **THEN** coverage report is uploaded to Codecov
- **AND** Codecov displays coverage badge
- **AND** PR comments show coverage changes

### Requirement: Coverage Tracking Tooling
The workspace SHALL provide coverage tracking using `cargo-tarpaulin` configured in workspace metadata.

#### Scenario: Coverage report generation
- **GIVEN** cargo-tarpaulin is installed
- **WHEN** developer runs `cargo tarpaulin --workspace`
- **THEN** coverage report is generated in `target/tarpaulin/`
- **AND** HTML report is viewable in browser
- **AND** XML report is generated for Codecov upload

#### Scenario: Coverage script available
- **GIVEN** a workspace with coverage configured
- **WHEN** developer runs `./scripts/coverage.sh`
- **THEN** coverage is calculated for all workspace crates
- **AND** report is saved to `target/coverage/`
- **AND** command exits with appropriate exit code

### Requirement: Development Dependencies
The workspace SHALL specify testing and coverage tools as development dependencies in workspace and individual crate manifests.

#### Scenario: Mocking framework available
- **GIVEN** cargo dependencies are resolved
- **WHEN** developer writes `use mockall::mock;`
- **THEN** code compiles without errors
- **AND** `mockall` crate is available in dev environment

#### Scenario: HTTP mocking available
- **GIVEN** cargo dependencies are resolved
- **WHEN** developer writes `use wiremock::MockServer;`
- **THEN** code compiles without errors
- **AND** `wiremock` crate is available in dev environment

#### Scenario: Snapshot testing available
- **GIVEN** cargo dependencies are resolved
- **WHEN** developer writes `insta::assert_snapshot!(value);`
- **THEN** code compiles without errors
- **AND** `insta` crate is available in dev environment

### Requirement: Test Documentation
The workspace SHALL provide comprehensive documentation for running tests locally, understanding coverage reports, and contributing to test suite.

#### Scenario: README includes testing section
- **GIVEN** a repository with README.md
- **WHEN** README is viewed
- **THEN** it includes "Testing" section
- **AND** it documents `cargo test --workspace`
- **AND** it documents `cargo tarpaulin` for coverage
- **AND** it includes coverage badge

#### Scenario: Contributing guide includes testing expectations
- **GIVEN** a repository with CONTRIBUTING.md
- **WHEN** CONTRIBUTING is viewed
- **THEN** it includes section on testing
- **AND** it describes when to add tests
- **AND** it describes test organization (unit vs integration)
- **AND** it references TEST_STRATEGY.md

### Requirement: Coverage Targets
The workspace SHALL define tiered coverage targets to prioritize testing effort and ensure critical code paths are adequately tested.

#### Scenario: Critical module coverage targets
- **GIVEN** coverage report is generated
- **WHEN** critical modules are analyzed
- **THEN** `tools/src/fs.rs` has 90%+ coverage
- **AND** `tools/src/cmd.rs` has 90%+ coverage
- **AND** `core/src/agent.rs` has 90%+ coverage
- **AND** `core/src/session.rs` has 90%+ coverage

#### Scenario: High priority module coverage targets
- **GIVEN** coverage report is generated
- **WHEN** high-priority modules are analyzed
- **THEN** `provider/src/openai.rs` has 80%+ coverage
- **AND** `server/src/lib.rs` has 80%+ coverage
- **AND** `core/src/session/context.rs` has 80%+ coverage

#### Scenario: Overall coverage target
- **GIVEN** coverage report is generated for entire workspace
- **WHEN** overall coverage is calculated
- **THEN** workspace has 80%+ overall coverage

#### Scenario: Medium priority module coverage targets
- **GIVEN** coverage report is generated
- **WHEN** medium-priority modules are analyzed
- **THEN** `cli-core/` has 70%+ coverage
- **AND** `common/src/` has 70%+ coverage
- **AND** `client/` has 70%+ coverage

#### Scenario: Low priority module coverage targets
- **GIVEN** coverage report is generated
- **WHEN** low-priority modules are analyzed (TUI)
- **THEN** `tui/` has 60%+ coverage (logic only)
- **AND** focus is on state machine, not rendering
