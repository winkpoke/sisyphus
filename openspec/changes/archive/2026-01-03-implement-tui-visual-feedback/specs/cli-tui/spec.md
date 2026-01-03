# cli-tui Specification Delta

## ADDED Requirements

### Requirement: Semantic Visual Theme
The TUI SHALL use a semantic color palette to distinguish message types and states.

#### Scenario: Message coloring
- **Given** the transcript contains messages from different roles
- **When** the TUI renders the transcript
- **Then** User messages SHALL be rendered in Soft Blue
- **And** Assistant messages SHALL be rendered in Lavender or Mint
- **And** System messages SHALL be rendered in Muted Grey

### Requirement: Visual Feedback for Actions
The TUI SHALL provide ephemeral visual feedback for user actions.

#### Scenario: Copy to clipboard toast
- **Given** the user has selected a message in the transcript
- **When** the user presses the copy hotkey (e.g., 'c')
- **Then** the message content SHALL be copied to the system clipboard
- **And** a temporary "Toast" notification (e.g., "✓ Copied") SHALL appear on screen
- **And** the notification SHALL automatically disappear after a short duration (e.g., 2 seconds)

### Requirement: Streaming State Indication
The TUI SHALL visually indicate when a message is actively streaming or generating.

#### Scenario: Streaming cursor
- **Given** the assistant is generating a response
- **When** the TUI renders the incomplete message
- **Then** a blinking cursor or spinner SHALL be displayed at the end of the message content
- **And** the indicator SHALL be removed once the message generation is complete
