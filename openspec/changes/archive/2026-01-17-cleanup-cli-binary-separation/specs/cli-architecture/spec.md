## MODIFIED Requirements

### Requirement: Modular CLI Architecture
The CLI codebase MUST be organized into modular components to ensure maintainability and testability.

#### Scenario: Code Structure
- **Given** CLI source code
- **Then** `crates/cli/src/main.rs` MUST only handle argument parsing and delegation to cli-core
- **And** binary crate MUST be a thin entry point with minimal logic
- **And** all business logic, REPL implementation, and command handling MUST be encapsulated in `crates/cli-core`
- **And** there MUST be no duplicate code between `crates/cli` and `crates/cli-core`
- **And** interactive UI MUST support multiple frontends (REPL and TUI) behind a selection mechanism

## RENAMED Requirements
- FROM: `crates/cli-lib`
- TO: `crates/cli-core`
- FROM: `sisyphus-cli-lib` (package name)
- TO: `sisyphus-cli-core` (package name)
