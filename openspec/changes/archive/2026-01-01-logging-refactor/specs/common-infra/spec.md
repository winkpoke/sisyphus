# common-infra Specification Delta

## ADDED Requirements

### Requirement: Event Bus Logging
The system MUST provide a dedicated mechanism to log all system events from the Event Bus using the structured logging system.

#### Scenario: Logging System Events
Given the system is running
When a `SystemEvent` is published to the `EventBus`
Then it is logged with the appropriate log level (Info, Debug, Error) and structured fields.
