## ADDED Requirements
### Requirement: Mode-Aware Logging Defaults
The system MUST provide different default logging levels based on execution mode: INFO for server mode, ERROR for interactive CLI modes (REPL, TUI, msg). Users MAY override defaults using the `RUST_LOG` environment variable.

#### Scenario: Server mode uses INFO level
- **WHEN** user runs `sisyphus serve`
- **THEN** system initializes logging with INFO level
- **AND** all informational messages are displayed (startup, requests, events)

#### Scenario: Interactive modes use ERROR level
- **WHEN** user runs `sisyphus repl`, `sisyphus tui`, or `sisyphus msg`
- **THEN** system initializes logging with ERROR level
- **AND** only error messages are displayed
- **AND** informational/debug logs are suppressed

#### Scenario: User overrides default logging level
- **GIVEN** user wants more verbose logging
- **WHEN** user sets `RUST_LOG=info` environment variable
- **THEN** system respects the `RUST_LOG` setting
- **AND** displays logs at the specified level regardless of mode

#### Scenario: Background server logs suppressed in interactive modes
- **GIVEN** user runs REPL or TUI mode
- **WHEN** CLI spawns a background server subprocess
- **THEN** subprocess stderr is piped and consumed silently
- **AND** server logs do not clutter interactive user output
