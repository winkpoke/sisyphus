## REMOVED Requirements
### Requirement: CLI Headless Server Management
**Reason**: Implicit server spawning behavior is now explicitly stated under individual interface requirements (REPL/TUI/msg) with clear server ownership semantics. The new structure provides more granular control over server lifecycle while maintaining the implicit spawn behavior when `--attach` is not provided.

### Requirement: CLI Remote Connection
**Reason**: Remote connection is now an optional `--attach` flag on REPL/TUI/msg commands, not a standalone subcommand. This provides better composability (choose interface + choose connection mode).

## ADDED Requirements

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

## MODIFIED Requirements

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
