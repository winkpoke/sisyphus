## ADDED Requirements

### Requirement: Session Tracks Current Agent
The Session entity SHALL track the identity of the Agent responsible for executing its chat turns.

#### Scenario: Session Stores Agent ID
- **WHEN** a new session is created by the SessionManager
- **THEN** the Session MUST store an `agent_id` field representing the associated Agent
- **AND** this field MUST be included in the Session's serialized representation used by the server API.

#### Scenario: Agent ID is Stable During Turn
- **GIVEN** an existing session `S1` with `agent_id` `A1`
- **WHEN** the Agent begins processing a chat turn for `S1`
- **THEN** the `agent_id` used for that turn MUST remain `A1` for the duration of the turn
- **AND** any concurrent request to change `S1`'s `agent_id` MUST be rejected according to the server API.

