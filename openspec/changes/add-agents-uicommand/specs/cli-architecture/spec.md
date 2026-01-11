## ADDED Requirements

### Requirement: /agents UiCommand lists available agents and current session agent
The CLI REPL SHALL provide a `/agents` UiCommand that displays all available agents from the server and indicates the currently active agent for the active session.

#### Scenario: List agents with no arguments
- **GIVEN** a CLI connected to a server with configured Agents `A1`, `A2`, `A3`
- **AND** the CLI has an active session `S1` associated with Agent `A2`
- **WHEN** the user inputs `/agents` with no arguments
- **THEN** the CLI MUST call `GET /api/v1/sessions/S1` to fetch session metadata including `agent_id`
- **AND** the CLI MUST call `GET /api/v1/agents` to fetch available agents
- **AND** the CLI MUST display a list of all available agents with their metadata (id, name, description, model)
- **AND** the CLI MUST visually indicate which agent is currently active for the session

#### Scenario: List agents when server returns empty list
- **GIVEN** a CLI connected to a server with no configured agents
- **WHEN** the user inputs `/agents` with no arguments
- **THEN** the CLI MUST display a message indicating no agents are available

### Requirement: /agents UiCommand changes session agent
The CLI REPL SHALL support switching the active agent for the active session via the `/agents` UiCommand with an agent ID argument.

#### Scenario: Change agent with valid agent ID
- **GIVEN** a CLI connected to a server with available Agents `A1` and `A2`
- **AND** the CLI has an active session `S1` associated with Agent `A1`
- **WHEN** the user inputs `/agents A2`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "A2" }`
- **AND** the CLI MUST display a success message confirming the agent change
- **AND** subsequent chat requests for the session MUST use Agent `A2`

#### Scenario: Change agent with invalid agent ID
- **GIVEN** a CLI connected to a server with available Agents `A1` and `A2`
- **AND** the CLI has an active session `S1` associated with Agent `A1`
- **WHEN** the user inputs `/agents invalid_agent`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "invalid_agent" }`
- **AND** the CLI MUST display an error message indicating the agent ID is not found
- **AND** the session MUST remain associated with its previous agent

#### Scenario: Prevent agent change while session is busy
- **GIVEN** a CLI connected to a server
- **AND** the CLI has an active session `S1`
- **WHEN** the user inputs `/agents A2`
- **THEN** the CLI MUST call `PUT /api/v1/sessions/S1/agent`
- **AND** if the server returns a deterministic 4xx error indicating the session is busy
- **THEN** the CLI MUST display an error message indicating the agent cannot be changed while the session is busy
- **AND** the session MUST remain associated with its previous agent

### Requirement: /agents UiCommand error handling
The CLI SHALL handle errors gracefully when the `/agents` command encounters API failures or unexpected responses.

#### Scenario: Handle server error when listing agents
- **GIVEN** a CLI connected to a server
- **WHEN** the user inputs `/agents`
- **AND** the call to `GET /api/v1/agents` or `GET /api/v1/sessions/:id` fails or returns an error response
- **THEN** the CLI MUST display an error message describing the failure
- **AND** the CLI MUST NOT crash or exit

#### Scenario: Handle server error when changing agent
- **GIVEN** a CLI connected to a server
- **WHEN** the user inputs `/agents A1`
- **AND** the call to `PUT /api/v1/sessions/S1/agent` fails or returns an error response
- **THEN** the CLI MUST display an error message describing the failure
- **AND** the session MUST remain associated with its previous agent
