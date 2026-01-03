## ADDED Requirements

### Requirement: Optional reasoning request controls
The system SHALL support optional reasoning request controls in a provider-agnostic form.

#### Scenario: Reasoning defaults are off
- **GIVEN** no reasoning settings are configured
- **WHEN** a completion request is constructed
- **THEN** the provider payload MUST NOT include any reasoning-specific fields

#### Scenario: Reasoning is enabled with normalized settings
- **GIVEN** reasoning is configured with `mode=on` and `effort=high`
- **WHEN** a completion request is constructed
- **THEN** the provider MUST attempt to map these settings into its outbound payload

### Requirement: Provider-agnostic request overrides
The system SHALL allow provider-agnostic JSON request overrides to be merged into the outbound provider payload.

#### Scenario: Overrides apply after standard fields
- **GIVEN** a completion request includes request overrides
- **WHEN** the provider constructs its JSON payload
- **THEN** the overrides MUST be deep-merged after the standard Sisyphus fields
- **AND** the overrides MUST take precedence on key conflicts

### Requirement: Reasoning outputs are mapped safely
The system SHALL support mapping provider-specific reasoning output into a safe summary channel.

#### Scenario: Summary is available without exposing raw reasoning
- **GIVEN** a provider response contains provider-specific reasoning output
- **WHEN** the system maps the response into its internal message model
- **THEN** it MUST be able to populate a `reasoning_summary` representation
- **AND** it MUST NOT require exposing raw chain-of-thought by default

