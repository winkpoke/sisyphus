## MODIFIED Requirements

### Requirement: Command Palette
The TUI SHALL provide a command palette for slash commands with keyboard navigation.

The command palette MUST present a merged list of:
- server-registered slash commands
- client-local slash commands registered by the TUI

#### Scenario: Palette includes local and remote commands
- **GIVEN** the TUI has registered at least one client-local command
- **AND** the server reports at least one server-registered slash command
- **WHEN** the user opens the command palette
- **THEN** the palette SHALL present both local and server-registered commands

#### Scenario: Palette selection dispatches via unified router
- **GIVEN** the command palette is open
- **WHEN** the user selects a command and submits it
- **THEN** the TUI SHALL dispatch it through the unified slash-command dispatch interface
- **AND** it SHALL execute client-local commands locally
- **AND** it SHALL forward server-registered commands to the server

