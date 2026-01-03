# cli-tui Specification

## Purpose
TBD - created by archiving change add-cli-tui-foundation. Update Purpose after archive.
## Requirements
### Requirement: TUI Availability and Fallback
The CLI SHALL provide an interactive TUI for chat sessions when stdin and stdout are attached to a TTY.

#### Scenario: TUI runs in a terminal
- **Given** the user runs `sisyphus` in an interactive terminal
- **When** the chat UI starts
- **Then** the CLI SHALL initialize terminal modes required for the TUI
- **And** the CLI SHALL render an interactive multi-pane UI

#### Scenario: Non-TTY fallback
- **Given** stdin is not a terminal or stdout is not a terminal
- **When** the chat UI starts
- **Then** the CLI SHALL not enable raw terminal modes
- **And** the CLI SHALL fall back to a non-TUI interaction mode

### Requirement: Terminal Mode Safety
The TUI MUST restore the terminal to a usable state on normal exit and on panic.

#### Scenario: Overlay usage remains safe
- **Given** the TUI is running with overlays active
- **When** the user exits
- **Then** the terminal MUST be restored to a usable state

### Requirement: Single Event Loop Rendering
The TUI SHALL process terminal input events and backend events through a single owner event loop.

#### Scenario: Streaming does not disrupt input
- **Given** the user is typing in the input composer
- **When** the backend emits streaming assistant output
- **Then** the TUI SHALL update the transcript progressively
- **And** the user’s in-progress input MUST remain unchanged

### Requirement: Progressive Transcript Rendering
The TUI SHALL render a transcript that supports progressive updates while the agent is responding.

#### Scenario: Assistant message streams into the transcript
- **Given** the user submits a message
- **When** assistant output arrives in multiple streamed chunks
- **Then** the transcript SHALL display the assistant message as it grows
- **And** the transcript SHALL not duplicate or reorder streamed chunks

### Requirement: Transcript Scrolling Semantics
The TUI SHALL support transcript scrolling with an explicit “stick to bottom” mode.

#### Scenario: Stick to bottom while receiving output
- **Given** the transcript is at the bottom
- **When** new transcript lines are appended
- **Then** the view SHALL remain at the bottom

#### Scenario: Manual scroll disables stickiness
- **Given** the user scrolls upward in the transcript
- **When** new transcript lines are appended
- **Then** the view SHALL remain anchored to the user’s scroll position
- **And** the UI SHALL indicate that new content is available off-screen

### Requirement: Resize Reflow
The TUI MUST reflow wrapped transcript content when the terminal width changes.

#### Scenario: Terminal resize recalculates wrapping
- **Given** the transcript contains wrapped lines
- **When** the terminal width changes
- **Then** the transcript rendering MUST reflow to the new width
- **And** the transcript MUST remain readable without truncated mid-grapheme output

### Requirement: Command Palette
The TUI SHALL provide a command palette for slash commands with keyboard navigation.

#### Scenario: Palette opens on slash
- **Given** the user is focused on the input composer
- **When** the user types `/`
- **Then** the TUI SHALL display a palette of available commands
- **And** the user SHALL be able to navigate the list with arrow keys
- **And** selecting a command SHALL insert it into the composer

#### Scenario: Palette filters commands
- **Given** the palette is open
- **When** the user types additional characters
- **Then** the palette SHALL filter commands by prefix match

### Requirement: Overlays and Pager
The TUI SHALL support overlays for help and errors, and a pager for long content.

#### Scenario: Help overlay
- **Given** the TUI is running
- **When** the user requests help
- **Then** the TUI SHALL show help in an overlay without printing to stdout

#### Scenario: Long content uses pager
- **Given** the TUI needs to display content longer than the available viewport
- **When** the user opens the content
- **Then** the TUI SHALL show it in a pager UI with scrolling

### Requirement: Transcript Selection and Copy
The TUI SHALL allow selecting transcript content and copying it for external use.

#### Scenario: Keyboard selection and copy
- **Given** the transcript contains content
- **When** the user enters selection mode and selects a range
- **Then** the TUI SHALL copy the selected text to the clipboard when supported
- **And** otherwise provide a fallback copy mechanism within the UI

### Requirement: Status Indicators and Key Hints
The TUI SHALL provide unobtrusive status indicators and key hints for discoverability.

#### Scenario: New content indicator while scrolled
- **Given** the user has scrolled away from the bottom
- **When** new transcript content arrives
- **Then** the UI SHALL indicate that new content is available off-screen

### Requirement: Permission Request Prompt
The TUI SHALL render `PermissionRequest` events as a first-class permission prompt instead of raw event JSON.

#### Scenario: Permission request opens an overlay
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** a `PermissionRequest` event is received
- **THEN** the TUI SHALL display an overlay describing the request
- **AND** it SHALL include `operation`, `tool_name`, and `call_id`

#### Scenario: Permission request indicates the agent is blocked
- **GIVEN** a permission request overlay is visible
- **WHEN** the user reads the overlay
- **THEN** the UI SHALL indicate that tool execution is blocked pending user action

### Requirement: Permission Approval Overlay Actions
When the TUI receives a `PermissionRequest` event, it SHALL present a permission overlay with Approve and Deny actions.

#### Scenario: Overlay opens on PermissionRequest
- **GIVEN** the TUI is running
- **WHEN** the backend emits a `PermissionRequest` event
- **THEN** the TUI SHALL show an overlay that includes `operation`, `tool_name`, and `call_id`
- **AND** the overlay SHALL indicate that the agent is blocked pending a decision

#### Scenario: Approve resumes the assistant turn
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **WHEN** the user selects Approve
- **THEN** the TUI SHALL submit an approval decision to the server
- **AND** the TUI SHALL append the resulting assistant response to the transcript
- **AND** the overlay SHALL close

#### Scenario: Deny resumes the assistant turn
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **WHEN** the user selects Deny
- **THEN** the TUI SHALL submit a denial decision to the server
- **AND** the TUI SHALL append the resulting assistant response to the transcript
- **AND** the overlay SHALL close

