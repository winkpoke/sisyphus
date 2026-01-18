# common-infra Specification

## Purpose
Defines shared infrastructure components used across all Sisyphus crates, including typed event bus for system events, configuration management, structured logging, and common data models for LLM interactions.
## Requirements
### Requirement: Configuration System
The system MUST load configuration from default values, a `sisyphus.toml` file, and Environment Variables.

#### Scenario: Loading Config
Given a `sisyphus.toml` with `server.port = 3000`
And an environment variable `SISYPHUS_SERVER_PORT=4000`
When the config is loaded
Then `server.port` should be `4000` (Env vars take precedence).

### Requirement: Structured Logging
The system MUST emit structured logs (JSON) for machine consumption and pretty logs for human reading.

#### Scenario: Log Output
Given the application is running
When a log event occurs
Then it includes the `session_id` in the context
And it is output in the configured format (JSON or Pretty).

### Requirement: Type-Safe Event Bus
The system MUST provide a typed, topic-based event bus for broadcasting system events efficiently. It MUST distribute events only to subscribers that have requested them (by topic), while supporting global auditing.

The event bus MUST broadcast `EventEnvelope<SystemEvent>` values (not raw `SystemEvent`) so consumers can rely on stable envelope metadata.

#### Scenario: Event broadcasting uses envelopes
- **GIVEN** a subscriber to the event bus
- **WHEN** an `AgentStateChanged` event is published
- **THEN** the subscriber SHALL receive an `EventEnvelope<SystemEvent>` containing the correct `AgentStateChanged` payload

#### Scenario: Specific subscription remains topic-based
- **GIVEN** an `EventBus` with a subscriber for `MessageReceived`
- **WHEN** a `ToolExecuted` event is published
- **THEN** the `MessageReceived` subscriber MUST NOT be woken up

#### Scenario: Global subscription receives all envelopes
- **GIVEN** an `EventBus` with a `subscribe_all` listener
- **WHEN** any event is published
- **THEN** the listener MUST receive an envelope for that event

#### Scenario: Raw global subscription receives all envelopes
- **GIVEN** an `EventBus` with a `subscribe_raw` listener
- **WHEN** any event is published
- **THEN** the listener MUST receive an envelope for that event

### Requirement: Event Bus Logging
The system MUST provide a dedicated mechanism to log all system events from the Event Bus using the structured logging system.

#### Scenario: Logging system events includes envelope metadata
- **GIVEN** the system is running
- **WHEN** a `SystemEvent` is published to the `EventBus`
- **THEN** it is logged with the appropriate log level (Info, Debug, Error) and structured fields
- **AND** the structured fields MUST include the envelope `id` and `timestamp_ms`

### Requirement: Internationalization Support
The system MUST support multiple languages for user-facing output.

#### Scenario: Default Language
Given the configuration does not specify a language
When the application starts
Then the language should default to English

#### Scenario: Chinese Language Support
Given the configuration specifies "zh-CN"
When the application starts
Then the output messages should be in Chinese

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

### Requirement: Event envelopes
The system MUST wrap each `SystemEvent` in an `EventEnvelope` that carries stable metadata for ordering and logging.

#### Scenario: Envelope includes stable metadata
- **GIVEN** a `SystemEvent` is published
- **WHEN** a subscriber receives the published item
- **THEN** the received item MUST include an `id` and `timestamp_ms`
- **AND** it MUST include the original `SystemEvent` payload

