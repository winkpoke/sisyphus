## MODIFIED Requirements

### Requirement: Start a new session via command
The system SHALL support starting a new session via an explicit session-creation API.

Session creation MUST produce a new session id and an empty session context.

#### Scenario: Create new session produces empty context
- **GIVEN** an existing session `S1` with prior context
- **WHEN** a new session is created via the session manager
- **THEN** the new session `S2` MUST have an empty message history
- **AND** the new session `S2` MUST have an empty tool-result history

### Requirement: Clear history via command
The system SHALL support clearing an existing session context via a dedicated clear operation.

#### Scenario: Clear operation clears context
- **GIVEN** an existing session with prior context
- **WHEN** the clear operation is executed for that session
- **THEN** the session context MUST be cleared before the next completion request

## ADDED Requirements

### Requirement: Session context mutations are not driven by SlashCommands
The system SHALL NOT require SlashCommands to mutate session lifecycle state.

#### Scenario: SlashCommand expansion does not clear context
- **GIVEN** a session with prior context
- **WHEN** the user executes a SlashCommand that expands to prompt text
- **THEN** the session context MUST remain intact unless an explicit session operation is invoked

