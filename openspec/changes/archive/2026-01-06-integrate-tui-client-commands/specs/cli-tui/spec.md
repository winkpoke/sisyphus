## ADDED Requirements

### Requirement: Functional TUI Commands
The TUI local commands MUST perform actual state changes in the application and backend.

#### Scenario: New Session Command
- **Given** the TUI is running
- **When** the user executes `/new`
- **Then** the CLI SHALL call the backend to create a new session
- **And** the CLI SHALL update the active session ID
- **And** the transcript SHALL be cleared and show a "Started new session" system message

#### Scenario: Clear History Command
- **Given** the transcript has content
- **When** the user executes `/clear`
- **Then** the transcript SHALL be cleared
- **And** the active session SHALL remain unchanged
- **And** if the TUI is connected to a backend session, the CLI SHALL clear the backend session history

#### Scenario: Exit Command
- **Given** the TUI is running
- **When** the user executes `/exit` or `/quit`
- **Then** the application SHALL terminate gracefully

### Requirement: UI Commands Do Not Require SystemEvent Extensions
UI-scoped commands (e.g., `/new`, `/clear`, `/exit`, `/quit`, `/debug`) MUST be implemented using TUI-local state transitions and instructions.

#### Scenario: UI-only effects are not encoded as SystemEvent
- **Given** the TUI executes a UI-scoped command
- **When** the TUI updates local state or triggers a client API call
- **Then** the TUI MUST NOT require adding new `SystemEvent` variants for these UI-only effects

### Requirement: UI Command Routing Uses Shared Slash Parsing
The TUI MUST identify slash command name and arguments using the shared core slash command parser so client routing matches server semantics.

#### Scenario: Quoted args are preserved for routing
- **Given** the user enters a slash command with quoted args
- **When** the TUI parses the input
- **Then** the command name and arguments SHALL match the server-side parser’s semantics
