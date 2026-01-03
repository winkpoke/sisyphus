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
- **THEN** the provider SHALL map these settings into its outbound payload when supported

#### Scenario: Unsupported reasoning settings are ignored safely
- **GIVEN** reasoning is configured with `mode=on` and `effort=high`
- **AND** the selected provider does not support a reasoning control surface
- **WHEN** a completion request is constructed
- **THEN** the provider MUST NOT include any reasoning-specific fields
- **AND** the request MUST still be sent successfully

### Requirement: Provider-agnostic request overrides
The system SHALL allow provider-agnostic JSON request overrides to be merged into the outbound provider payload.

#### Scenario: Overrides apply after standard fields
- **GIVEN** a completion request includes request overrides
- **WHEN** the provider constructs its JSON payload
- **THEN** the overrides MUST be deep-merged after the standard Sisyphus fields
- **AND** the overrides MUST take precedence on key conflicts

#### Scenario: Overrides cannot modify reserved request keys
- **GIVEN** a completion request includes request overrides
- **AND** the overrides contain a reserved request key such as `messages` or `tools`
- **WHEN** the provider constructs its JSON payload
- **THEN** the provider MUST ignore reserved-key overrides
- **AND** the payload MUST preserve Sisyphus-standard values for reserved keys

### Requirement: Reasoning outputs are mapped safely
The system SHALL support mapping provider-specific reasoning output into a safe summary channel.

#### Scenario: Summary is available without exposing raw reasoning
- **GIVEN** a provider response contains provider-specific reasoning output
- **WHEN** the system maps the response into its internal message model
- **THEN** it MUST be able to populate a `reasoning_summary` representation
- **AND** it MUST NOT require exposing raw chain-of-thought by default

#### Scenario: Raw reasoning is optional and gated
- **GIVEN** a provider response contains raw reasoning output
- **WHEN** the system maps the response into its internal message model
- **THEN** it MAY populate a `reasoning_raw` representation
- **AND** `reasoning_raw` MUST NOT be required to support `reasoning_summary`
