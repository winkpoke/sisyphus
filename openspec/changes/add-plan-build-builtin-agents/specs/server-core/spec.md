## ADDED Requirements

### Requirement: Default Server Runtime Exposes Built-in Agents
When started with a default configuration, the server SHALL expose both built-in Agents via the Agent discovery API.

#### Scenario: Agent discovery includes plan and build
- **GIVEN** a running server started with default configuration
- **WHEN** a client sends `GET /api/v1/agents`
- **THEN** the response MUST include an Agent object with id `plan`
- **AND** the response MUST include an Agent object with id `build`

### Requirement: Default Agent is Plan
When no agent id is provided, the server MUST default sessions to the Plan agent.

#### Scenario: Create session defaults to plan
- **GIVEN** a running server started with default configuration
- **WHEN** a client sends `POST /api/v1/sessions` with no body or without `agent_id`
- **THEN** the server MUST treat the session as associated with agent id `plan`
