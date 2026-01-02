# cli-architecture Specification Deltas

## ADDED Requirements

### Requirement: Interactive Command Menu
The CLI SHALL provide an interactive menu for selecting commands when the user initiates a command sequence.

#### Scenario: Triggering the Menu
- **Given** the CLI is in the idle chat state
- **When** the user presses the `/` key
- **Then** the CLI MUST immediately display a vertical menu of available commands
- **And** the `/` character MUST be inserted into the input buffer

#### Scenario: Navigating the Menu
- **Given** the command menu is visible
- **When** the user presses the Up or Down arrow keys
- **Then** the selection highlight MUST move accordingly
- **And** the input buffer MUST NOT change until selection is confirmed

#### Scenario: Selecting a Command
- **Given** a command is highlighted in the menu
- **When** the user presses Enter
- **Then** the command name MUST be appended to the `/` in the input buffer
- **And** the menu MUST disappear
- **And** the cursor MUST be positioned after the command name

### Requirement: Dynamic Command Discovery
The CLI SHALL fetch the list of available commands from the connected server to populate the menu.

#### Scenario: Populating the Menu
- **Given** a connected server with custom agent commands (e.g., "analyze-code")
- **And** builtin commands (e.g., "help", "exit")
- **When** the command menu is triggered
- **Then** both builtin and custom commands MUST be listed
- **And** each item SHOULD display its name and a brief description
