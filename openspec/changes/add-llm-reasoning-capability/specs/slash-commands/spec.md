## ADDED Requirements

### Requirement: /think toggles reasoning summary output
The system SHALL provide a `/think` slash command that toggles reasoning summary output for the current session.

#### Scenario: /think enables summary output
- **GIVEN** the current session has reasoning summary output disabled
- **WHEN** the user executes `/think`
- **THEN** reasoning summary output SHALL become enabled for that session

#### Scenario: /think disables summary output
- **GIVEN** the current session has reasoning summary output enabled
- **WHEN** the user executes `/think`
- **THEN** reasoning summary output SHALL become disabled for that session

### Requirement: /think is safe by default
The `/think` command MUST NOT enable raw reasoning output.

#### Scenario: /think does not enable raw reasoning
- **WHEN** the user executes `/think`
- **THEN** raw reasoning output MUST remain disabled
