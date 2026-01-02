# cli-architecture

## ADDED Requirements

### Requirement: Modular CLI Architecture
The CLI codebase MUST be organized into modular components to ensure maintainability and testability.

#### Scenario: Code Structure
- **Given** the CLI source code
- **Then** `main.rs` MUST only handle argument parsing and dispatching
- **And** business logic MUST be encapsulated in `commands/` modules
- **And** UI logic MUST be encapsulated in `ui/` modules
- **And** agent initialization MUST be isolated in a bootstrap module
