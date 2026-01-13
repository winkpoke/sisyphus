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

#### Scenario: Event Broadcasting
Given a subscriber to the event bus
When an `AgentStateChanged` event is published
Then the subscriber receives the event with the correct payload.

#### Scenario: Specific Subscription
Given an `EventBus` with a subscriber for `MessageReceived`
When a `ToolExecuted` event is published
Then the `MessageReceived` subscriber MUST NOT be woken up

#### Scenario: Global Subscription
Given an `EventBus` with a `subscribe_all` listener
When any event is published
Then the listener MUST receive the event

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

