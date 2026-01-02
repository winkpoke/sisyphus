# cli-architecture Specification

## Purpose
To provide a modular, maintainable, and extensible CLI client for Sisyphus that supports headless server management, remote connections, and event streaming.
## Requirements
### Requirement: CLI Headless Server Management
The CLI SHALL manage a local server instance when no remote server is specified.
#### Scenario: CLI Spawns Server on Demand
- **Given** the user runs `sisyphus` without arguments
- **When** the command starts
- **Then** a new `sisyphus serve` process is spawned
- **And** the server uses the port defined in the configuration file or the default code-defined port
- **And** the CLI connects to this local server
- **And** the server is terminated when the CLI exits

### Requirement: CLI Remote Connection
The CLI SHALL support connecting to an external server instance via a URL argument.
#### Scenario: CLI Connects to Existing Server
- **Given** a running sisyphus server on port 4000
- **When** the user runs `sisyphus attach http://localhost:4000`
- **Then** the CLI connects to the existing server
- **And** no new server process is spawned
- **And** the server remains running when the CLI exits

### Requirement: Event Streaming
The CLI SHALL consume real-time events from the server to display agent activity.

#### Scenario: Real-time Event Streaming
- **Given** a connected CLI session
- **When** the user sends a message
- **Then** the CLI receives updates (thoughts, text chunks) via SSE
- **And** renders them progressively to the terminal
- **And** progressive rendering MUST remain stable under streaming (no flicker-driven input loss)

### Requirement: Startup Banner Display
The CLI MUST display a branded startup banner upon initialization.

#### Scenario: Launching the CLI
Given the user runs the `sisyphus` command
When the application starts
Then an orange "SISYPHUS" ASCII logo is displayed
And a summary box showing the version, current model, and working directory is shown
And a usage tip is displayed below the box.

### Requirement: Startup Banner Data Accuracy
The startup banner MUST display accurate runtime information derived from the application state.

#### Scenario: Displaying Active Configuration
Given the application is configured with model "gpt-4-turbo"
And the application version is "0.1.0"
When the banner is rendered
Then the displayed model field MUST be "gpt-4-turbo"
And the displayed version MUST match "0.1.0"
And the displayed directory MUST match the current working directory.

### Requirement: Banner Styling
The startup banner MUST use specific colors and formatting.

#### Scenario: Visual Elements
Given the banner is being rendered
Then the logo must be colored orange (approx. #E35728)
And the information box must use Unicode box-drawing characters
And the model and directory paths must be clearly legible.

### Requirement: Modular CLI Architecture
The CLI codebase MUST be organized into modular components to ensure maintainability and testability.

#### Scenario: Code Structure
- **Given** the CLI source code
- **Then** `main.rs` MUST only handle argument parsing and dispatching
- **And** business logic MUST be encapsulated in `commands/` modules
- **And** UI logic MUST be encapsulated in `ui/` modules
- **And** the interactive UI MUST support multiple frontends (REPL and TUI) behind a selection mechanism

### Requirement: REPL Session Switching
The CLI REPL SHALL adopt new session IDs returned by the server after lifecycle commands.

#### Scenario: REPL Updates Session ID After /new
Given the CLI is connected to a server using session id `S1`
When the user sends "/new"
And the server returns a chat response indicating a new session id `S2`
Then the CLI SHALL store `S2` as the active session id
And subsequent chat requests SHALL use `S2`

