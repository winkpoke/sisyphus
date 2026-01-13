## MODIFIED Requirements

### Requirement: Modular CLI Architecture
The CLI codebase MUST be organized into modular components to ensure maintainability and testability.

#### Scenario: Code Structure
- **Given** the CLI source code
- **Then** `main.rs` MUST only handle argument parsing and dispatching
- **And** business logic MUST be encapsulated in `commands/` modules
- **And** UI logic MUST be encapsulated in `ui/` modules (for REPL) and a separate `tui/` crate (for TUI)
- **And** the interactive UI MUST support multiple frontends (REPL and TUI) behind a selection mechanism
- **And** the TUI MUST be provided as an optional build feature via `cargo build --features tui`

#### Scenario: CLI-only Build Excludes TUI Dependencies
- **Given** the CLI source code
- **When** the CLI is built without the `tui` feature
- **Then** the build MUST NOT include TUI-specific dependencies (ratatui, crossterm, arboard)
- **And** the binary MUST be smaller than a build with TUI enabled
- **And** the REPL MUST still function correctly

#### Scenario: TUI Build Includes TUI Crate
- **Given** the CLI source code
- **When** the CLI is built with the `tui` feature enabled
- **Then** the build MUST include the separate `tui` crate
- **And** the TUI-specific dependencies MUST be available
- **And** the TUI MUST be accessible via the `--tui` flag
