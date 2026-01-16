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
- **Given** CLI source code
- **Then** `crates/cli/src/main.rs` MUST only handle argument parsing and delegation to cli-lib
- **And** binary crate MUST be a thin entry point with minimal logic
- **And** all business logic, REPL implementation, and command handling MUST be encapsulated in `crates/cli-lib`
- **And** there MUST be no duplicate code between `crates/cli` and `crates/cli-lib`
- **And** interactive UI MUST support multiple frontends (REPL and TUI) behind a selection mechanism

### Requirement: REPL Session Switching
The CLI REPL SHALL adopt new session IDs returned by the server after UiCommand-driven lifecycle operations.

#### Scenario: REPL Updates Session ID After /new
Given the CLI is connected to a server using session id `S1`
When the user runs the UiCommand `/new`
And the CLI creates a new session via `POST /api/v1/sessions`
Then the CLI SHALL store the new session id `S2` as the active session id
And subsequent chat requests SHALL use `S2`

### Requirement: UiCommand routing and precedence
The CLI client SHALL route slash-prefixed user input through a UiCommand router before sending input to the server.

If the command name matches a known UiCommand, the CLI MUST execute it locally.

#### Scenario: UiCommand takes precedence over SlashCommand name collisions
- **GIVEN** the CLI registers a UiCommand `/exit`
- **AND** the server reports a SlashCommand named `/exit`
- **WHEN** the user inputs `/exit`
- **THEN** the CLI MUST execute the UiCommand locally
- **AND** it MUST NOT send the input to the chat endpoint

### Requirement: Escaping leading slash for chat
The CLI client SHALL support escaping a leading slash so users can send literal slash text.

#### Scenario: Double slash sends literal slash
- **GIVEN** the CLI is connected to a server
- **WHEN** the user inputs `//help`
- **THEN** the CLI MUST send `/help` as normal chat text
- **AND** it MUST NOT treat it as a command

### Requirement: /agents UiCommand lists available agents and current session agent
The CLI REPL SHALL provide a `/agents` UiCommand that displays all available agents from the server and indicates the currently active agent for the active session.

#### Scenario: List agents with no arguments
- **GIVEN** a CLI connected to a server with configured Agents `A1`, `A2`, `A3`
- **AND** the CLI has an active session `S1` associated with Agent `A2`
- **WHEN** the user inputs `/agents` with no arguments
- **THEN** the CLI MUST call `GET /api/v1/sessions/S1` to fetch session metadata including `agent_id`
- **AND** the CLI MUST call `GET /api/v1/agents` to fetch available agents
- **AND** the CLI MUST display a list of all available agents with their metadata (id, name, description, model)
- **AND** the CLI MUST visually indicate which agent is currently active for the session

#### Scenario: List agents when server returns empty list
- **GIVEN** a CLI connected to a server with no configured agents
- **WHEN** the user inputs `/agents` with no arguments
- **THEN** the CLI MUST display a message indicating no agents are available

### Requirement: /agents UiCommand changes session agent
The CLI REPL SHALL support switching the active agent for the active session via the `/agents` UiCommand with an agent ID argument.

#### Scenario: Change agent with valid agent ID
- **GIVEN** a CLI connected to a server with available Agents `A1` and `A2`
- **AND** the CLI has an active session `S1` associated with Agent `A1`
- **WHEN** the user inputs `/agents A2`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "A2" }`
- **AND** the CLI MUST display a success message confirming the agent change
- **AND** subsequent chat requests for the session MUST use Agent `A2`

#### Scenario: Change agent with invalid agent ID
- **GIVEN** a CLI connected to a server with available Agents `A1` and `A2`
- **AND** the CLI has an active session `S1` associated with Agent `A1`
- **WHEN** the user inputs `/agents invalid_agent`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "invalid_agent" }`
- **AND** the CLI MUST display an error message indicating the agent ID is not found
- **AND** the session MUST remain associated with its previous agent

#### Scenario: Prevent agent change while session is busy
- **GIVEN** a CLI connected to a server
- **AND** the CLI has an active session `S1`
- **WHEN** the user inputs `/agents A2`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent`
- **AND** if the server returns a deterministic 4xx error indicating the session is busy
- **THEN** the CLI MUST display an error message indicating the agent cannot be changed while the session is busy
- **AND** the session MUST remain associated with its previous agent

### Requirement: /agents UiCommand error handling
The CLI SHALL handle errors gracefully when the `/agents` command encounters API failures or unexpected responses.

#### Scenario: Handle server error when listing agents
- **GIVEN** a CLI connected to a server
- **WHEN** the user inputs `/agents`
- **AND** the call to `GET /api/v1/agents` or `GET /api/v1/sessions/:id` fails or returns an error response
- **THEN** the CLI MUST display an error message describing the failure
- **AND** the CLI MUST NOT crash or exit

#### Scenario: Handle server error when changing agent
- **GIVEN** a CLI connected to a server
- **WHEN** the user inputs `/agents A1`
- **AND** the call to `PUT /api/v1/sessions/S1/agent` fails or returns an error response
- **THEN** the CLI MUST display an error message describing the failure
- **AND** the session MUST remain associated with its previous agent

