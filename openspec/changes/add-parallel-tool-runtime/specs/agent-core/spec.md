# agent-core Delta Specification

## ADDED Requirements

### Requirement: Tool execution runtime
The system SHALL execute tool calls using a dedicated runtime that supports selective parallel execution.

#### Scenario: Runtime enforces execution modes
- **GIVEN** a completion response includes multiple tool calls
- **WHEN** the agent executes those tool calls
- **THEN** Parallel tools MAY execute concurrently
- **AND** Sequential tools MUST execute exclusively

