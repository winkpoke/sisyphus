# cli-architecture Specification

## Purpose
To provide a modular, maintainable, and extensible CLI client for Sisyphus that supports headless server management, remote connections, and event streaming.
## Requirements
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
  - **Then** `crates/cli/src/main.rs` MUST only handle argument parsing and delegation to cli-core
  - **And** binary crate MUST be a thin entry point with minimal logic
  - **And** all business logic, REPL implementation, and command handling MUST be encapsulated in `crates/cli-core`
  - **And** there MUST be no duplicate code between `crates/cli` and `crates/cli-core`
  - **And** command modules MUST be organized by interface type: `repl.rs`, `tui.rs`, `msg.rs`, `serve.rs`
  - **And** each command module MUST support optional `--attach <url>` flag for remote connections
  - **And** the `attach` functionality MUST NOT be a separate module but integrated into each command

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

### Requirement: Clean Interactive Mode Output
The CLI MUST spawn background server subprocesses with stderr piped and consumed silently to prevent log output from interfering with interactive user experience.

#### Scenario: REPL mode suppresses server logs
- **WHEN** user runs `sisyphus repl` without remote URL
- **THEN** CLI spawns a server subprocess with stderr piped
- **AND** server logs are consumed silently in background
- **AND** user sees only clean interactive REPL interface

#### Scenario: TUI mode suppresses server logs
- **WHEN** user runs `sisyphus tui` without remote URL
- **THEN** CLI spawns a server subprocess with stderr piped
- **AND** server logs are consumed silently in background
- **AND** user sees only clean TUI interface

#### Scenario: Server mode shows logs normally
- **WHEN** user runs `sisyphus serve` directly
- **THEN** server subprocess inherits stderr normally
- **AND** all logs are displayed for debugging

#### Scenario: Remote connection unaffected
- **WHEN** user runs interactive mode with `--attach` to remote server
- **THEN** no subprocess is spawned
- **AND** logging behavior is determined by remote server

### Requirement: Implicit Local Server Spawning
The CLI SHALL automatically spawn a local server instance when no `--attach` URL is provided for REPL, TUI, or msg commands.

#### Scenario: REPL spawns local server when not attached
- **Given** the user runs `sisyphus repl` without `--attach`
- **When** the command starts
- **Then** the CLI spawns a local server process on an available port
- **And** the CLI connects to this local server
- **And** the CLI takes ownership of the server (will stop it on exit)

#### Scenario: TUI spawns local server when not attached
- **Given** the user runs `sisyphus tui` without `--attach`
- **When** the command starts
- **Then** the CLI spawns a local server process on an available port
- **And** the CLI connects to this local server
- **And** the CLI takes ownership of the server (will stop it on exit)

#### Scenario: One-shot msg spawns local server when not attached
- **Given** the user runs `sisyphus msg "message"` without `--attach`
- **When** the command starts
- **Then** the CLI spawns a local server process on an available port
- **And** the CLI connects to this local server
- **And** the CLI takes ownership of the server (will stop it on exit)

#### Scenario: Remote connection does not spawn local server
- **Given** the user runs `sisyphus repl --attach http://localhost:4000`
- **When** the command starts
- **Then** the CLI does NOT spawn a local server process
- **And** the CLI connects to the existing remote server
- **And** the CLI does NOT take ownership (will leave server running on exit)

### Requirement: REPL Interface
The CLI SHALL provide a `repl` subcommand that starts an interactive Read-Eval-Print Loop for chat sessions.

#### Scenario: Start local REPL
- **Given** the user runs `sisyphus repl`
- **When** the command starts
- **Then** a local server is spawned if needed
- **And** the CLI connects to the local server
- **And** an interactive REPL session starts
- **And** the server is terminated when the REPL exits

#### Scenario: Start remote REPL
- **Given** a running sisyphus server on port 4000
- **When** the user runs `sisyphus repl --attach http://localhost:4000`
- **Then** the CLI connects to the existing server at the specified URL
- **And** no new server process is spawned
- **And** an interactive REPL session starts
- **And** the server remains running when the REPL exits

#### Scenario: REPL with malformed URL
- **Given** the user runs `sisyphus repl --attach not-a-url`
- **When** the command starts
- **Then** the CLI displays a clear error message indicating the URL is invalid
- **And** the CLI exits with a non-zero status code

#### Scenario: REPL with unreachable remote server
- **Given** no server is running on port 4000
- **When** the user runs `sisyphus repl --attach http://localhost:4000`
- **Then** the CLI displays a clear error message indicating the server is unreachable
- **And** the CLI exits with a non-zero status code

#### Scenario: REPL local server spawn failure
- **Given** the port allocation fails (all ports in use)
- **When** the user runs `sisyphus repl`
- **Then** the CLI displays a clear error message indicating server startup failed
- **And** the CLI exits with a non-zero status code

### Requirement: TUI Interface
The CLI SHALL provide a `tui` subcommand that starts an interactive Terminal User Interface for chat sessions.

#### Scenario: Start local TUI
- **Given** the user runs `sisyphus tui`
- **When** the command starts
- **Then** a local server is spawned if needed
- **And** the CLI connects to the local server
- **And** an interactive TUI session starts (requires `--features tui` to build)
- **And** the server is terminated when the TUI exits

#### Scenario: Start remote TUI
- **Given** a running sisyphus server on port 4000
- **When** the user runs `sisyphus tui --attach http://localhost:4000`
- **Then** the CLI connects to the existing server at the specified URL
- **And** no new server process is spawned
- **And** an interactive TUI session starts
- **And** the server remains running when the TUI exits

#### Scenario: TUI feature not enabled
- **Given** the CLI was built without the `tui` feature
- **When** the user runs `sisyphus tui`
- **Then** the CLI MUST display an error message indicating the TUI feature is not enabled
- **And** the CLI MUST exit with a non-zero status code

#### Scenario: TUI with malformed URL
- **Given** the user runs `sisyphus tui --attach not-a-url`
- **When** the command starts
- **Then** the CLI displays a clear error message indicating the URL is invalid
- **And** the CLI exits with a non-zero status code

#### Scenario: TUI with unreachable remote server
- **Given** no server is running on port 4000
- **When** the user runs `sisyphus tui --attach http://localhost:4000`
- **Then** the CLI displays a clear error message indicating the server is unreachable
- **And** the CLI exits with a non-zero status code

### Requirement: One-Shot Message
The CLI SHALL provide a `msg` subcommand for sending a single message to the agent and receiving the response, then exiting.

#### Scenario: Send one-shot local message
- **Given** the user runs `sisyphus msg "hello world"`
- **When** the command executes
- **Then** a local server is spawned if needed
- **And** a session is created
- **And** the message is sent to the agent
- **And** the CLI waits for and displays the complete response
- **And** the session and server are terminated after the response
- **And** the CLI exits with status code 0 on success

#### Scenario: Send one-shot remote message
- **Given** a running sisyphus server on port 4000
- **When** the user runs `sisyphus msg --attach http://localhost:4000 "hello world"`
- **Then** the CLI connects to the existing server at the specified URL
- **And** no new server process is spawned
- **And** a session is created on the remote server
- **And** the message is sent to the agent
- **And** the CLI waits for and displays the complete response
- **And** the session is terminated after the response
- **And** the server remains running
- **And** the CLI exits with status code 0 on success

#### Scenario: One-shot message with empty string
- **Given** the user runs `sisyphus msg ""` (empty message)
- **When** the command executes
- **Then** the CLI displays a clear error message indicating the message cannot be empty
- **And** no server is spawned
- **And** the CLI exits with a non-zero status code

#### Scenario: One-shot message with whitespace-only
- **Given** the user runs `sisyphus msg "   "` (only whitespace)
- **When** the command executes
- **Then** the CLI displays a clear error message indicating the message cannot be empty
- **And** no server is spawned
- **And** the CLI exits with a non-zero status code

#### Scenario: One-shot message with malformed URL
- **Given** the user runs `sisyphus msg --attach not-a-url "hello"`
- **When** the command executes
- **Then** the CLI displays a clear error message indicating the URL is invalid
- **And** no server is spawned
- **And** the CLI exits with a non-zero status code

#### Scenario: One-shot message with unreachable remote server
- **Given** no server is running on port 4000
- **When** the user runs `sisyphus msg --attach http://localhost:4000 "hello"`
- **Then** the CLI displays a clear error message indicating the server is unreachable
- **And** the CLI exits with a non-zero status code

#### Scenario: One-shot message with server timeout
- **Given** a running sisyphus server that takes too long to respond
- **When** the user runs `sisyphus msg "hello"`
- **Then** the CLI waits for the response (no timeout in current implementation)
- **And** when response completes, it is displayed
- **Note**: Future enhancement may add configurable timeout with exit code 2 for timeout

#### Scenario: One-shot message with streaming response
- **Given** the agent returns a long response that streams in chunks
- **When** the user runs `sisyphus msg "generate a list"`
- **Then** the CLI collects all streamed chunks until completion
- **And** the complete response is displayed at once after streaming finishes
- **And** the CLI detects response completion (e.g., via SSE end-of-stream signal)
- **And** the CLI exits cleanly after response is complete

### Requirement: Dedicated Server Mode
The CLI SHALL provide a `serve` subcommand for running the server in standalone mode without an interactive client.

#### Scenario: Start dedicated server
- **Given** the user runs `sisyphus serve --port 3000`
- **When** the command executes
- **Then** the server starts on the specified port (or default port if not specified)
- **And** the server runs until terminated
- **And** the server does not spawn any interactive client

### Requirement: CLI Command Testing
The system SHALL provide tests for CLI commands (repl, msg, serve, tui) to verify command execution, argument parsing, and user interaction.

#### Scenario: REPL command execution
- **GIVEN** a CLI with REPL command configured
- **WHEN** `sisyphus repl` is executed
- **THEN** REPL starts successfully
- **AND** user input is accepted
- **AND** commands are parsed correctly
- **AND** exit works (Ctrl+D or `quit`)

#### Scenario: MSG command one-shot message
- **GIVEN** a CLI with MSG command configured
- **WHEN** `sisyphus msg "test message"` is executed
- **THEN** message is sent to server
- **AND** response is printed to stdout
- **AND** CLI exits after response

#### Scenario: Serve command lifecycle
- **GIVEN** a CLI with SERVE command configured
- **WHEN** `sisyphus serve` is executed
- **THEN** server starts on configured port
- **AND** server runs until interrupted
- **AND** server shuts down gracefully on SIGTERM

#### Scenario: TUI command initialization
- **GIVEN** a CLI with TUI command configured
- **WHEN** `sisyphus tui` is executed
- **THEN** TUI interface starts successfully
- **AND** terminal is initialized
- **AND** user can interact via keyboard

#### Scenario: Command help display
- **GIVEN** a CLI with any command
- **WHEN** `--help` flag is provided
- **THEN** help text is displayed
- **AND** all subcommands are listed
- **AND** usage examples are shown

### Requirement: CLI Configuration Testing
The system SHALL provide tests for CLI configuration loading, environment variables, and profile switching.

#### Scenario: Environment variable configuration
- **GIVEN** a CLI without config file
- **AND** environment variable `LLM_PROVIDER` is set
- **WHEN** CLI is started
- **THEN** provider is loaded from environment
- **AND** configuration takes precedence over defaults

#### Scenario: Profile configuration loading
- **GIVEN** a CLI with multiple config profiles
- **WHEN** CLI is started with `--profile development`
- **THEN** development profile is loaded
- **AND** profile-specific settings are applied
- **AND** profile overrides default config

#### Scenario: Config file validation
- **GIVEN** a CLI with invalid config file
- **WHEN** CLI is started
- **THEN** appropriate error is displayed
- **AND** error indicates config issue
- **AND** CLI exits with non-zero status

### Requirement: Server Integration Testing
The system SHALL provide tests for CLI-server integration to verify connection, authentication, and communication.

#### Scenario: Server connection establishment
- **GIVEN** a CLI with server URL configured
- **WHEN** CLI attempts to connect to server
- **THEN** connection succeeds if server is running
- **AND** appropriate error if server is down
- **AND** connection timeout is handled

#### Scenario: Authentication with API key
- **GIVEN** a CLI with API key configured
- **WHEN** CLI sends request to server
- **THEN** API key is included in headers
- **AND** server accepts authenticated request
- **AND** API key is not logged

#### Scenario: Message transmission to server
- **GIVEN** a CLI connected to server
- **WHEN** user types message in REPL
- **THEN** message is sent via HTTP
- **AND** response is received and displayed
- **AND** conversation history is maintained

#### Scenario: Permission prompt handling
- **GIVEN** a CLI with REPL and permission prompt configured
- **WHEN** tool execution requires approval
- **THEN** prompt is displayed to user
- **AND** user input blocks until decision
- **AND** decision is sent to server
- **AND** CLI resumes after approval

### Requirement: REPL State Management
The system SHALL provide tests for REPL state (history, mode, active session) to ensure REPL behavior is correct.

#### Scenario: REPL command history
- **GIVEN** a REPL session
- **WHEN** user enters multiple commands
- **THEN** command history is maintained
- **AND** arrow keys navigate history
- **AND** history persists across sessions (if configured)

#### Scenario: REPL mode switching
- **GIVEN** a REPL with command modes
- **WHEN** user switches to different mode
- **THEN** mode indicator is updated
- **AND** mode-specific behavior is activated
- **AND** previous mode state is saved

#### Scenario: Active session tracking
- **GIVEN** a REPL connected to server
- **WHEN** session is created or switched
- **THEN** active session ID is tracked
- **AND** session metadata is displayed
- **AND** messages route to correct session

### Requirement: CLI Error Handling
The system SHALL provide tests for CLI error scenarios to ensure user-friendly error messages and graceful failure handling.

#### Scenario: Network error handling
- **GIVEN** a CLI attempting server connection
- **AND** network is unreachable
- **WHEN** connection fails
- **THEN** appropriate error message is displayed
- **AND** CLI offers retry option
- **AND** CLI remains in recoverable state

#### Scenario: Server error handling
- **GIVEN** a CLI connected to server
- **WHEN** server returns 5xx error
- **THEN** error message is displayed
- **AND** error includes status code and details
- **AND** CLI does not crash

#### Scenario: Invalid command handling
- **GIVEN** a REPL session
- **WHEN** user enters unrecognized command
- **THEN** error message indicates command not found
- **AND** help suggestion is displayed
- **AND** REPL continues accepting input

#### Scenario: TUI keyboard event processing
- **GIVEN** a TUI interface active
- **WHEN** user presses keyboard keys (arrows, enter, escape)
- **THEN** events are processed correctly
- **AND** state updates appropriately
- **AND** no event is lost

#### Scenario: TUI terminal resize handling
- **GIVEN** a TUI interface active
- **WHEN** terminal window is resized
- **THEN** layout is recalculated
- **AND** components reposition correctly
- **AND** no rendering errors occur

#### Scenario: TUI screen component positioning
- **GIVEN** a TUI interface with multiple components
- **WHEN** TUI is rendered
- **THEN** components are positioned correctly
- **AND** no overlapping occurs
- **AND** layout is consistent

