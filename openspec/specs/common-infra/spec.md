# common-infra Specification

## Purpose
TBD - created by archiving change scaffold-phase-1. Update Purpose after archive.
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
The system MUST provide a typed event bus for broadcasting system events.

#### Scenario: Event Broadcasting
Given a subscriber to the event bus
When an `AgentStateChanged` event is published
Then the subscriber receives the event with the correct payload.

### Requirement: Event Bus Logging
The system MUST provide a dedicated mechanism to log all system events from the Event Bus using the structured logging system.

#### Scenario: Logging System Events
Given the system is running
When a `SystemEvent` is published to the `EventBus`
Then it is logged with the appropriate log level (Info, Debug, Error) and structured fields.

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

