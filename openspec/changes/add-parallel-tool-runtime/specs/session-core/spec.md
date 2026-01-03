# session-core Delta Specification

## MODIFIED Requirements

### Requirement: Preserve multi-step turn ordering
The system SHALL preserve deterministic ordering of multi-step turns, including tool exchanges.

#### Scenario: Tool result ordering is deterministic
- **GIVEN** a single assistant message includes multiple tool calls
- **WHEN** those tools are executed (potentially in parallel)
- **THEN** tool result messages are appended in the same order as the tool calls

