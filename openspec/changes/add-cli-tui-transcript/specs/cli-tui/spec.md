## MODIFIED Requirements

### Requirement: Single Event Loop Rendering
The TUI SHALL process terminal input events and backend events through a single owner event loop.

#### Scenario: Streaming does not disrupt input
- **Given** the user is typing in the input composer
- **When** the backend emits streaming assistant output
- **Then** the TUI SHALL update the transcript progressively
- **And** the user’s in-progress input MUST remain unchanged

## ADDED Requirements

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

