## ADDED Requirements

### Requirement: Reasoning summary visibility toggle
The TUI SHALL allow toggling the visibility of reasoning summaries.

#### Scenario: Summaries are hidden by default
- **GIVEN** the TUI starts in a new session
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_summary"` is received
- **THEN** the TUI MUST hide it by default

#### Scenario: Toggle shows reasoning summaries
- **GIVEN** the user enables reasoning summary visibility
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_summary"` is received
- **THEN** the TUI SHALL display it as a system transcript entry

#### Scenario: /think toggles summary visibility
- **GIVEN** the TUI is running
- **WHEN** the user executes `/think`
- **THEN** the TUI SHALL toggle reasoning summary visibility

### Requirement: Debug-only raw reasoning display
If raw reasoning content is ever made available to the TUI, it MUST only be displayable when debug mode is enabled.

#### Scenario: Debug gate is enforced
- **GIVEN** debug mode is disabled
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_raw"` is received
- **THEN** the TUI MUST NOT render raw reasoning content
