## ADDED Requirements

### Requirement: Reasoning is not persisted by default
The agent MUST NOT persist raw reasoning content into the session context by default.

#### Scenario: Raw reasoning is excluded from context
- **GIVEN** reasoning is enabled
- **AND** regardless of reasoning storage configuration
- **WHEN** the agent appends the assistant response to the session context
- **THEN** raw reasoning MUST NOT be included in stored messages

#### Scenario: Reasoning summary storage is explicit
- **GIVEN** the provider returns a `reasoning_summary`
- **AND** reasoning storage is configured as `store=none`
- **WHEN** the agent appends the assistant response to the session context
- **THEN** the reasoning summary MUST NOT be included in stored messages

### Requirement: Reasoning summary emission is opt-in
When configured, the agent SHALL emit a reasoning summary as a system-visible transcript entry.

#### Scenario: Summary is emitted as a system message
- **GIVEN** reasoning exposure is configured as `summary`
- **WHEN** the provider returns a reasoning summary for an assistant response
- **THEN** the agent SHALL publish a `MessageReceived` event with `role = "system"`
- **AND** the event MUST include `kind = "reasoning_summary"`

#### Scenario: Summary is not emitted when disabled
- **GIVEN** reasoning exposure is configured as `none`
- **WHEN** the provider returns a reasoning summary for an assistant response
- **THEN** the agent MUST NOT publish any reasoning summary transcript entry

### Requirement: Session-scoped reasoning exposure overrides
The agent SHALL support a session-scoped override that can enable or disable reasoning summary emission at runtime.

#### Scenario: /think enables summary emission for the session
- **GIVEN** the session reasoning exposure is `none`
- **WHEN** the user executes `/think`
- **THEN** the session reasoning exposure SHALL become `summary`

#### Scenario: /think disables summary emission for the session
- **GIVEN** the session reasoning exposure is `summary`
- **WHEN** the user executes `/think`
- **THEN** the session reasoning exposure SHALL become `none`

### Requirement: Auto mode is deterministic
When reasoning is configured with `mode=auto`, the agent SHALL enable reasoning only under deterministic, testable conditions.

#### Scenario: Auto enables after tool usage begins
- **GIVEN** reasoning mode is configured as `auto`
- **AND** the session context contains at least one Tool message
- **WHEN** the agent constructs the next completion request
- **THEN** the agent SHALL request reasoning from the provider

#### Scenario: Auto remains off in non-tool sessions
- **GIVEN** reasoning mode is configured as `auto`
- **AND** the session context contains no Tool messages
- **WHEN** the agent constructs a completion request
- **THEN** the agent MUST NOT request reasoning from the provider
