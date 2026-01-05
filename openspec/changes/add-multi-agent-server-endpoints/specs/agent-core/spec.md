## ADDED Requirements

### Requirement: Expose Agent Metadata via Server API
The system SHALL expose Agent configuration metadata, including model settings, via the server Agent discovery API.

#### Scenario: Agent Metadata Reflects Config
- **GIVEN** an Agent configured with a specific name, description, mode, permissions, and model settings
- **WHEN** the server returns that Agent via `GET /api/v1/agents` or `GET /api/v1/agents/:id`
- **THEN** the returned metadata MUST include the configured name, description, and model
- **AND** it MUST NOT expose internal-only fields that are not part of the Agent configuration contract.

