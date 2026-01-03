## ADDED Requirements

### Requirement: Reasoning is not persisted by default
The agent MUST NOT persist raw reasoning content into the session context by default.

#### Scenario: Raw reasoning is excluded from context
- **GIVEN** reasoning is enabled
- **AND** exposure is not set to store raw reasoning
- **WHEN** the agent appends the assistant response to the session context
- **THEN** raw reasoning MUST NOT be included in stored messages

### Requirement: Reasoning summary emission is opt-in
When configured, the agent SHALL emit a reasoning summary as a system-visible transcript entry.

#### Scenario: Summary is emitted as a system message
- **GIVEN** reasoning exposure is configured as `summary`
- **WHEN** the provider returns a reasoning summary for an assistant response
- **THEN** the agent SHALL publish a `MessageReceived` event with `role = "system"`
- **AND** the content MUST be prefixed with `Reasoning summary:`

#### Scenario: Summary is not emitted when disabled
- **GIVEN** reasoning exposure is configured as `none`
- **WHEN** the provider returns a reasoning summary for an assistant response
- **THEN** the agent MUST NOT publish any reasoning summary transcript entry

