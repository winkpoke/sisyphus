## ADDED Requirements

### Requirement: /think is a reserved UiCommand name
The system SHALL reserve `/think` as a UiCommand name and MUST NOT allow registering it as a SlashCommand.

#### Scenario: Custom SlashCommand colliding with /think is rejected
- **GIVEN** `/think` is a reserved UiCommand name
- **AND** a command directory contains `think.md`
- **WHEN** the agent starts
- **THEN** the system MUST NOT register `/think` as a SlashCommand
