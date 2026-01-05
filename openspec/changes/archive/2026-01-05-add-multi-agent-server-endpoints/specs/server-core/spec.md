## ADDED Requirements

### Requirement: Agent Discovery API
The server SHALL provide endpoints to discover available Agents and their model settings.

#### Scenario: List Agents
- **GIVEN** a running server with one or more configured Agents
- **WHEN** a client sends `GET /api/v1/agents`
- **THEN** the server MUST return a JSON array of Agent objects
- **AND** each object MUST include `id`, `name`, `description`, and `model` fields.

#### Scenario: Get Agent by ID
- **GIVEN** a running server with a configured Agent `A1`
- **WHEN** a client sends `GET /api/v1/agents/A1`
- **THEN** the server MUST return a JSON object with `id`, `name`, `description`, and `model` for `A1`
- **AND** the server MUST return a suitable 4xx error if `A1` does not exist.

### Requirement: Session Creation with Agent Selection
The server SHALL allow sessions to be created with an explicitly selected Agent.

#### Scenario: Create Session with Default Agent
- **GIVEN** a running server with a configured default Agent `A_default`
- **WHEN** a client sends `POST /api/v1/sessions` with no body or without `agent_id`
- **THEN** the server MUST create a new session associated with `A_default`
- **AND** the response MUST include the session id and the `agent_id` `A_default`.

#### Scenario: Create Session with Specific Agent
- **GIVEN** a running server with configured Agents `A1` and `A2`
- **WHEN** a client sends `POST /api/v1/sessions` with body `{ "agent_id": "A2" }`
- **THEN** the server MUST create a new session associated with `A2`
- **AND** the response MUST include the session id and the `agent_id` `A2`
- **AND** the server MUST return a suitable 4xx error if `agent_id` does not match any configured Agent.

### Requirement: Session Agent Management API
The server SHALL provide endpoints to inspect and change the Agent associated with an existing session.

#### Scenario: Get Session Agent
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **WHEN** a client sends `GET /api/v1/sessions/S1`
- **THEN** the response MUST include the `agent_id` `A1` in the session metadata.

#### Scenario: Change Session Agent
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **AND** the session is not currently processing a chat turn
- **WHEN** a client sends `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "A2" }`
- **THEN** the server MUST update `S1` to be associated with `A2`
- **AND** the response MUST confirm the new `agent_id` `A2`.

#### Scenario: Prevent Agent Change While Busy
- **GIVEN** a running server and an existing session `S1` whose status is `Busy`
- **WHEN** a client sends `PUT /api/v1/sessions/S1/agent` with any body
- **THEN** the server MUST NOT change the session's `agent_id`
- **AND** the server MUST return a deterministic 4xx error indicating the session is busy.

### Requirement: Chat Response Includes Agent Metadata
The chat response SHALL include the effective Agent identity used for the turn.

#### Scenario: Chat Response Agent ID
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **WHEN** a client sends `POST /api/v1/sessions/S1/chat`
- **THEN** the response MUST include the `agent_id` `A1`
- **AND** the `model` field in the response MUST reflect the model configured for `A1`.

