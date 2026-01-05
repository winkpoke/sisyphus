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
The system MUST provide a typed event bus for broadcasting system events as `EventEnvelope<SystemEvent>`.

#### Scenario: Event broadcasting uses envelopes
- **GIVEN** a subscriber to the event bus
- **WHEN** an `AgentStateChanged` system event is published
- **THEN** the subscriber receives an envelope containing the event with the correct payload

### Requirement: Event Bus Logging
The system MUST provide a dedicated mechanism to log all system events from the Event Bus using the structured logging system.

#### Scenario: Logging system events includes envelope metadata
- **GIVEN** the system is running
- **WHEN** a `SystemEvent` is published to the `EventBus`
- **THEN** it is logged with structured fields that include the envelope `id` and `timestamp_ms`

