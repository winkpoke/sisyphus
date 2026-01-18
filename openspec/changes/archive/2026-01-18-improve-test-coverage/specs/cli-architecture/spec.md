# cli-architecture Specification (Delta)

## ADDED Requirements

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
