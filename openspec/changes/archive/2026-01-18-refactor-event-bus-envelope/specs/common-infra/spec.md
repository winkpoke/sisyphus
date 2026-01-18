# common-infra Delta Specification

## ADDED Requirements

### Requirement: Event envelopes
The system MUST wrap each `SystemEvent` in an `EventEnvelope` that carries stable metadata for ordering and logging.

#### Scenario: Envelope includes stable metadata
- **GIVEN** a `SystemEvent` is published
- **WHEN** a subscriber receives the published item
- **THEN** the received item MUST include an `id` and `timestamp_ms`
- **AND** it MUST include the original `SystemEvent` payload

## MODIFIED Requirements

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
