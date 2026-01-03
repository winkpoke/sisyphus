# Tooling Delta Specification

## ADDED Requirements

### Requirement: Tool execution modes
The system SHALL assign each registered tool an execution mode: Parallel or Sequential.

#### Scenario: Default execution mode is Sequential
- **GIVEN** a tool is registered without an explicit execution mode
- **WHEN** the tool is executed
- **THEN** it is executed in Sequential mode

### Requirement: Selective parallel execution
The system SHALL allow concurrent execution of Parallel tools while ensuring Sequential tools execute exclusively.

#### Scenario: Parallel tools can run concurrently
- **GIVEN** two Parallel tool calls are scheduled
- **WHEN** the system executes them
- **THEN** they MAY execute at the same time

#### Scenario: Sequential tool blocks all other tools
- **GIVEN** a Sequential tool call is scheduled
- **WHEN** the system begins executing it
- **THEN** no other tool call executes until it completes

## MODIFIED Requirements

### Requirement: Glob Tool
The system SHALL treat `glob` as a Parallel tool.

#### Scenario: Glob executes under parallel mode
- **GIVEN** `glob` is registered
- **WHEN** the agent schedules multiple `glob` tool calls
- **THEN** the runtime MAY execute those calls concurrently

### Requirement: Grep Tool
The system SHALL treat `grep` as a Parallel tool.

#### Scenario: Grep executes under parallel mode
- **GIVEN** `grep` is registered
- **WHEN** the agent schedules multiple `grep` tool calls
- **THEN** the runtime MAY execute those calls concurrently
