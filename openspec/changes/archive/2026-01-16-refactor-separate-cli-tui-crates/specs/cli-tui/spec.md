## ADDED Requirements

### Requirement: TUI Crate Isolation
The TUI MUST be implemented in a separate crate (`crates/tui`) from the CLI to enable independent testing, dependency management, and feature selection.

#### Scenario: TUI Crate Has Own Dependencies
- **Given** the workspace includes a `tui` crate
- **Then** the TUI crate MUST declare its own dependencies (ratatui, crossterm, arboard) in `Cargo.toml`
- **And** the CLI crate MUST NOT include these dependencies when the `tui` feature is disabled

#### Scenario: TUI Crate Depends on CLI
- **Given** the TUI crate implementation
- **Then** the TUI crate MUST depend on the CLI crate for shared utilities (banner, completer) and client management
- **And** the CLI crate MUST NOT depend on the TUI crate directly (only via optional feature)

#### Scenario: TUI Code is Self-Contained
- **Given** the TUI crate source code
- **When** examining the crate structure
- **Then** the TUI crate MUST contain all TUI-specific code (MVU architecture, components, event handling)
- **And** the TUI crate MUST NOT contain CLI-specific code (argument parsing, command routing, server management)
