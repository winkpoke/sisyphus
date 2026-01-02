## MODIFIED Requirements

### Requirement: Modular CLI Architecture
The CLI codebase MUST be organized into modular components to ensure maintainability and testability.

#### Scenario: Code Structure
- **Given** the CLI source code
- **Then** `main.rs` MUST only handle argument parsing and dispatching
- **And** business logic MUST be encapsulated in `commands/` modules
- **And** UI logic MUST be encapsulated in `ui/` modules
- **And** the interactive UI MUST support multiple frontends (REPL and TUI) behind a selection mechanism

### Requirement: Event Streaming
The CLI SHALL consume real-time events from the server to display agent activity.

#### Scenario: Real-time Event Streaming
- **Given** a connected CLI session
- **When** the user sends a message
- **Then** the CLI receives updates (thoughts, text chunks) via SSE
- **And** renders them progressively to the terminal
- **And** progressive rendering MUST NOT corrupt the interactive UI (no interleaved prints)

