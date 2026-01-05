## MODIFIED Requirements

### Requirement: Command Palette
The TUI SHALL provide a command palette for both UiCommands and SlashCommands with keyboard navigation.

#### Scenario: Palette opens on slash
- **Given** the user is focused on the input composer
- **When** the user types `/` as the first character
- **And** the next character typed is not `/`
- **Then** the TUI SHALL display a palette of available commands
- **And** the palette MUST include UiCommands and SlashCommands
- **And** selecting a command SHALL insert it into the composer

#### Scenario: Double slash does not open the palette
- **Given** the user is focused on the input composer
- **When** the user types `//` as the first two characters
- **Then** the TUI MUST NOT open the command palette

#### Scenario: Palette filters commands
- **Given** the palette is open
- **When** the user types additional characters
- **Then** the palette SHALL filter commands by prefix match

## ADDED Requirements

### Requirement: UiCommands may call server endpoints
The TUI SHALL execute UiCommands locally and MAY call server endpoints when server state must change.

#### Scenario: /clear clears history via endpoint
- **GIVEN** an active session `S1`
- **WHEN** the user runs the UiCommand `/clear`
- **THEN** the TUI MUST call the server clear endpoint for `S1`
- **AND** the transcript MUST be cleared locally after the server acknowledges success

### Requirement: /help uses merged command discovery
The TUI SHALL render help from a merged view of UiCommands and SlashCommands.

#### Scenario: Help shows both command kinds
- **GIVEN** the server reports at least one custom SlashCommand
- **AND** the TUI provides at least one UiCommand
- **WHEN** the user requests help
- **THEN** the help content MUST include both UiCommands and SlashCommands
- **AND** each entry MUST indicate whether it is a UiCommand or SlashCommand
