## MODIFIED Requirements

### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

#### Scenario: Discoverable command listing
- **Given** the user is using the interactive TUI
- **When** the user requests command discovery (e.g., typing `/`)
- **Then** the UI SHALL present available slash commands without requiring the user to memorize names

