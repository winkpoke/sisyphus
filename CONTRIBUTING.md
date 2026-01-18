# Contributing to Sisyphus

Thank you for your interest in contributing to Sisyphus!

## Development Workflow

### Setup

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/sisyphus.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Check formatting: `cargo fmt --all`
7. Run linter: `cargo clippy --workspace --all-targets -- -D warnings`
8. Push to your fork
9. Open a pull request

## Testing

### Running Tests Locally

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific crate
cargo test -p sisyphus_core
```

### Coverage

We maintain a target of 80% overall code coverage with 90%+ for critical security modules.

Generate coverage locally:

```bash
./scripts/coverage.sh
```

View coverage report:

```bash
open target/tarpaulin/index.html
```

### Continuous Integration

CI/CD pipeline runs on every push and pull request:

1. **Test Job**: Builds, tests, lints, and checks formatting
   - Matrix builds for Rust stable and beta
   - Caches Cargo dependencies for faster builds
   - Fails on clippy warnings or formatting issues

2. **Coverage Job**: Generates and uploads coverage to Codecov
   - Runs after test job succeeds
   - Uploads coverage report for visualization

See `.github/workflows/test.yml` for CI/CD configuration.

## Code Style

- Follow Rust conventions: `cargo fmt` for formatting
- Use `clippy`: `cargo clippy --workspace --all-targets -- -D warnings`
- Keep functions focused and small
- Add tests for new functionality
- Document public APIs with rustdoc comments

## Testing Guidelines

### Test Organization

- **Unit tests**: Place in `#[cfg(test)]` modules within source files
- **Integration tests**: Place in `tests/` directories at crate level
- **Test fixtures**: Place in `tests/fixtures/` directories
- **Golden files**: Place in `tests/golden/` directories
- **Snapshots**: Managed automatically by `insta` (review with `cargo insta review`)

### Test Coverage Targets

- **Critical (90%+)**: `tools/src/fs.rs`, `tools/src/cmd.rs`, `core/src/agent.rs`, `core/src/session.rs`
- **High (80%+)**: `provider/src/openai.rs`, `server/src/lib.rs`, `core/src/session/context.rs`
- **Medium (70%+)**: `cli-core/`, `common/src/`, `client/`
- **Low (60%+)**: `tui/` (logic only, not rendering)

### Writing Good Tests

- Test both success and failure paths
- Test edge cases and error conditions
- Test security-critical code paths thoroughly
- Use descriptive test names
- Keep tests independent and fast
- Mock external dependencies (LLM providers, file system)

## Pull Request Process

1. Update related documentation
2. Ensure all tests pass
3. Update coverage if needed
4. Write clear commit messages
5. Reference related issues in PR description
6. Keep PRs focused and small

## Questions?

Feel free to open an issue for clarification before starting work on a feature.
