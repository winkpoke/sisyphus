## MODIFIED Requirements

### Requirement: Agent Configuration
The system SHALL support configuring Agents with strictly typed metadata including id, name, mode, permissions, and model settings.

#### Scenario: Load Valid Config with id
- **WHEN** a valid agent configuration JSON/TOML is loaded with both `id` and `name` fields
- **THEN** system correctly parses id, name, mode, and permission rules
- **AND** `id` is used as stable routing identifier for registry lookups
- **AND** `name` is used as display label for API responses

#### Scenario: Agent routing uses stable IDs
- **GIVEN** an Agent with `id="plan"` and `name="Plan Agent"`
- **WHEN** agent is registered in the registry
- **THEN** registry key is `"plan"` (not `"Plan Agent"`)
- **AND** API endpoints look up agent by `"plan"` (not by name)
- **AND** changing `name` to `"Planning Agent"` does not break existing clients using `"plan"`

## ADDED Requirements

### Requirement: Built-in Agent Definitions
The system SHALL provide two built-in agents with stable identifiers and distinct tool sets: a read-only `plan` agent for analysis, and an execution-capable `build` agent for implementation.

#### Scenario: Default agent is Plan agent
- **GIVEN** a new session is created without specifying an `agent_id`
- **WHEN** session initializes
- **THEN** it defaults to `plan` agent
- **AND** `plan` agent is read-only by tool exposure and permission policy

#### Scenario: Plan agent is read-only
- **GIVEN** `plan` agent is configured
- **WHEN** tools are registered on the plan agent
- **THEN** only read-only tools are registered (read_file, glob, grep)
- **AND** write_file and execute_command tools are NOT registered
- **AND** permission policy denies edit and command execution operations

#### Scenario: Agent discovery returns both agents
- **GIVEN** both `plan` and `build` agents are registered
- **WHEN** a client requests `GET /api/v1/agents`
- **THEN** response includes two agents with distinct IDs
- **AND** each agent includes: `id`, `name`, `description`, and `model`

#### Scenario: Agent selection by ID works
- **GIVEN** both agents are registered in the registry
- **WHEN** a client creates a session with `agent_id: "plan"`
- **THEN** session uses Plan agent
- **AND** subsequent tool calls are routed to Plan agent
- **AND** Plan agent cannot call write_file or execute_command (permission deny)

#### Scenario: Agent switching works mid-session
- **GIVEN** an active session using Build agent
- **WHEN** a client sends `PUT /api/v1/sessions/:id/agent` with `agent_id: "plan"`
- **THEN** session switches to Plan agent
- **AND** subsequent chat requests are routed to Plan agent
- **AND** Plan agent's tool restrictions apply immediately

### Requirement: Agent Identity Separation
The system SHALL separate agent routing identity (`id`) from display identity (`name`) to enable stable client integration.

#### Scenario: Display name changes do not break routing
- **GIVEN** an agent with `id="plan"` and `name="Plan Agent"`
- **WHEN** agent's display name is changed to `"Planning Agent"` in a future update
- **THEN** routing ID `"plan"` remains unchanged
- **AND** existing clients using `agent_id: "plan"` continue to work
- **AND** new API responses show updated display name

### Requirement: Tool Registration Validation
The system SHALL provide a helper function to register read-only tools with validation to prevent accidental registration of state-changing tools on Plan agent.

#### Scenario: Read-only tool registration validates allowed tools
- **GIVEN** Plan agent's `register_read_only_tools()` helper is called
- **WHEN** attempting to register `write_file` tool
- **THEN** registration fails with an error message indicating tool is not allowed
- **AND** tool is NOT added to agent
- **AND** error message includes list of allowed tool names

#### Scenario: Read-only tool registration accepts allowed tools
- **GIVEN** Plan agent's `register_read_only_tools()` helper is called
- **WHEN** attempting to register `read_file` tool
- **THEN** registration succeeds
- **AND** tool is available for agent to use

### Requirement: Shared LLM Provider
The system SHALL allow initializing multiple built-in agents derived from the same runtime LLM configuration.

#### Scenario: Multiple agents share one provider
- **GIVEN** an LLM provider is configured
- **WHEN** both Plan and Build agents are initialized
- **THEN** both agents use the same model configuration
