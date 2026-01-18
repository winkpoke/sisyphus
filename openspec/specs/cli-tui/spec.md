# cli-tui Specification

## Purpose
Defines the Terminal User Interface (TUI) for Sisyphus CLI, implementing a Model-View-Update (MVU) architectural pattern. The TUI provides interactive chat with progressive rendering, command palettes, agent selection, permission approvals, and real-time event streaming.
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

### Requirement: Layout Components
The TUI SHALL implement a full-screen layout consisting of a persistent Context Bar, a central Transcript area, and a unified Input/Status footer.

#### Scenario: Persistent Context Bar
- **Given** the TUI is running
- **Then** a Context Bar SHALL be displayed at the top of the screen
- **And** it SHALL display the application brand ("Sisyphus")
- **And** it SHALL display the current working directory
- **And** it SHALL display the active model name
- **And** it SHALL have a bottom border for visual separation

#### Scenario: Unified Input and Status Footer
- **Given** the TUI is running
- **Then** the bottom area SHALL contain the Input Composer and Status Indicator
- **And** the Input Composer SHALL occupy approximately 70% of the width
- **And** the Status Indicator SHALL occupy approximately 30% of the width
- **And** they SHALL be visually merged into a single footer row

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

### Requirement: Degraded Mode for Small Terminals
The TUI SHALL degrade gracefully when the terminal size is insufficient for the full layout.

#### Scenario: Minimum Size Detection
- **Given** the terminal size changes
- **When** the width is less than 80 columns OR the height is less than 24 rows
- **Then** the TUI SHALL switch to a degraded mode
- **And** the Context Bar SHALL be hidden or simplified
- **And** the Input/Status footer SHALL remain visible
- **And** the Transcript SHALL remain visible

#### Scenario: Critical Size Warning
- **Given** the terminal size is critically small (e.g., < 40x10)
- **Then** the TUI SHALL display a warning overlay
- **And** the warning SHALL request the user to resize the terminal

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

#### Scenario: Permission request does not spam raw JSON by default
- **GIVEN** the TUI is connected to the server event stream
- **AND** debug mode is disabled
- **WHEN** a `PermissionRequest` event is received via the SSE stream as an `EventEnvelope<SystemEvent>`
- **THEN** the TUI SHALL display the permission overlay
- **AND** the transcript SHALL NOT include the raw JSON payload for that event

### Requirement: Permission Approval Overlay Actions
When the TUI receives a `PermissionRequest` event, it SHALL present a permission overlay with Approve and Deny actions.

#### Scenario: Overlay opens on PermissionRequest
- **GIVEN** the TUI is running
- **WHEN** the backend emits a `PermissionRequest` event
- **THEN** the TUI SHALL show an overlay that includes `operation`, `tool_name`, and `call_id`
- **AND** the overlay SHALL indicate that the agent is blocked pending a decision

#### Scenario: Multiple permission requests are queued and actionable
- **GIVEN** the TUI is running
- **WHEN** the backend emits PermissionRequest events for call ids `C1` and `C2` in that order
- **THEN** the TUI MUST enqueue both requests
- **AND** the overlay MUST present `C1` before `C2`
- **AND** after `C1` is approved or denied, the overlay MUST present `C2`

#### Scenario: Approve advances the pending approval queue
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **AND** there is at least one other pending permission request
- **WHEN** the user selects Approve
- **THEN** the TUI SHALL submit an approval decision to the server
- **AND** the overlay MUST remain available to present the next pending request

#### Scenario: Deny advances the pending approval queue
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **AND** there is at least one other pending permission request
- **WHEN** the user selects Deny
- **THEN** the TUI SHALL submit a denial decision to the server
- **AND** the overlay MUST remain available to present the next pending request

### Requirement: Transcript Visuals
The transcript view MUST provide adequate whitespace, context, and visual distinction between message types.

#### Scenario: Message Blocks
- **Given** the transcript displays messages
- **Then** each message SHALL be rendered as a distinct block
- **And** each block SHALL have a header indicating the sender (User, Assistant, System)
- **And** the header SHALL include a timestamp

#### Scenario: Kind-Specific Headers
- **Given** a message block is rendered
- **Then** the header visuals SHALL differ based on the message kind
- **And** User messages SHALL use a specific separator style (e.g., "─")
- **And** Assistant messages SHALL use a specific separator style (e.g., "╌")
- **And** System messages SHALL use a specific separator style (e.g., "·")

#### Scenario: Dynamic Header
Given a session is active
Then the transcript block title should display the current Context Name or Session ID
Instead of the static text "Transcript".

#### Scenario: Content Padding
Given the transcript is displaying messages
Then there should be padding between the text and the block borders
To improve readability.

### Requirement: Human-readable backend event rendering
The TUI SHALL render backend SSE system events as concise, end-user-readable transcript entries by default.

#### Scenario: Unparseable event payload does not crash the UI
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an SSE message is received that cannot be parsed as an `EventEnvelope<SystemEvent>`
- **THEN** the TUI MUST remain responsive
- **AND** it SHALL append a minimal system message indicating an unparsed event was received

### Requirement: Debug toggle for raw backend event payloads
The TUI SHALL provide a local `/debug` command to toggle visibility of raw backend event payloads.

#### Scenario: Debug is disabled by default
- **GIVEN** the TUI starts in a new session
- **THEN** the TUI SHALL NOT display raw backend event payloads in the transcript

#### Scenario: /debug toggles raw payload visibility
- **GIVEN** the TUI is running and debug mode is disabled
- **WHEN** the user runs `/debug`
- **THEN** the TUI SHALL enable debug mode
- **AND** subsequent backend events SHALL include raw payload output

- **GIVEN** the TUI is running and debug mode is enabled
- **WHEN** the user runs `/debug`
- **THEN** the TUI SHALL disable debug mode
- **AND** subsequent backend events SHALL NOT include raw payload output

#### Scenario: /debug is discoverable in the command palette
- **GIVEN** the user opens the command palette
- **THEN** `/debug` SHALL appear in the available commands list

### Requirement: Raw payload redaction and truncation
When debug mode is enabled, raw backend event payloads MUST be displayed in a redacted and truncated form.

#### Scenario: Raw payload redacts sensitive fields
- **GIVEN** debug mode is enabled
- **AND** an SSE event payload contains sensitive values
- **WHEN** the TUI displays the raw payload
- **THEN** sensitive values MUST be replaced with a redaction marker

#### Scenario: Raw payload is truncated to a fixed maximum size
- **GIVEN** debug mode is enabled
- **AND** an SSE event payload exceeds the maximum raw display size
- **WHEN** the TUI displays the raw payload
- **THEN** the displayed payload MUST be truncated

### Requirement: Semantic Visual Theme
The TUI SHALL use a semantic color palette to distinguish message types and states.

#### Scenario: Message coloring
- **Given** the transcript contains messages from different roles
- **When** the TUI renders the transcript
- **Then** User messages SHALL be rendered in Soft Blue
- **And** Assistant messages SHALL be rendered in Lavender or Mint
- **And** System messages SHALL be rendered in Muted Grey

### Requirement: Accessibility Support
The TUI MUST support accessibility for users with color vision deficiencies by relying on structural and symbolic cues in addition to color.

#### Scenario: Colorblind-friendly separators
- **Given** messages are rendered in the transcript
- **Then** message kinds MUST be distinguishable by their header separator characters
- **And** relying solely on color for distinction is PROHIBITED

#### Scenario: Context Bar borders
- **Given** the Context Bar is displayed
- **Then** it MUST include a visible bottom border
- **And** the border MUST ensure separation from the transcript regardless of color contrast

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

### Requirement: TUI MVU Architecture
The TUI MUST be implemented using the Model-View-Update (MVU) architectural pattern.

#### Scenario: Separation of concerns
- **Given** the TUI codebase
- **Then** state mutations MUST be confined to an `update` module
- **And** UI rendering MUST be confined to a `ui` module
- **And** user inputs MUST be captured as `Action` enums before processing

#### Scenario: Component-based UI
- **Given** the TUI rendering logic
- **Then** distinct UI elements (Transcript, Input, Overlays) MUST be implemented as separate, reusable components

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

### Requirement: Functional TUI Commands
The TUI local commands MUST perform actual state changes in the application and backend.

#### Scenario: New Session Command
- **Given** the TUI is running
- **When** the user executes `/new`
- **Then** the CLI SHALL call the backend to create a new session
- **And** the CLI SHALL update the active session ID
- **And** the transcript SHALL be cleared and show a "Started new session" system message

#### Scenario: Clear History Command
- **Given** the transcript has content
- **When** the user executes `/clear`
- **Then** the transcript SHALL be cleared
- **And** the active session SHALL remain unchanged
- **And** if the TUI is connected to a backend session, the CLI SHALL clear the backend session history

#### Scenario: Exit Command
- **Given** the TUI is running
- **When** the user executes `/exit` or `/quit`
- **Then** the application SHALL terminate gracefully

### Requirement: UI Commands Do Not Require SystemEvent Extensions
UI-scoped commands (e.g., `/new`, `/clear`, `/exit`, `/quit`, `/debug`) MUST be implemented using TUI-local state transitions and instructions.

#### Scenario: UI-only effects are not encoded as SystemEvent
- **Given** the TUI executes a UI-scoped command
- **When** the TUI updates local state or triggers a client API call
- **Then** the TUI MUST NOT require adding new `SystemEvent` variants for these UI-only effects

### Requirement: UI Command Routing Uses Shared Slash Parsing
The TUI MUST identify slash command name and arguments using the shared core slash command parser so client routing matches server semantics.

#### Scenario: Quoted args are preserved for routing
- **Given** the user enters a slash command with quoted args
- **When** the TUI parses the input
- **Then** the command name and arguments SHALL match the server-side parser’s semantics

### Requirement: TUI Crate Isolation
The TUI MUST be implemented in a separate crate (`crates/tui`) from the CLI to enable independent testing, dependency management, and feature selection.

#### Scenario: TUI Crate Has Own Dependencies
- **Given** the workspace includes a `tui` crate
- **Then** the TUI crate MUST declare its own dependencies (ratatui, crossterm, arboard) in `Cargo.toml`
- **And** the CLI crate MUST NOT include these dependencies when the `tui` feature is disabled

#### Scenario: TUI Crate Depends on CLI
- **Given** the TUI crate implementation
- **Then** the TUI crate MUST depend on the CLI crate for shared utilities (banner, completer) and client management
- **And** the CLI crate MUST NOT depend on the TUI crate directly (only via optional feature)

#### Scenario: TUI Code is Self-Contained
- **Given** the TUI crate source code
- **When** examining the crate structure
- **Then** the TUI crate MUST contain all TUI-specific code (MVU architecture, components, event handling)
- **And** the TUI crate MUST NOT contain CLI-specific code (argument parsing, command routing, server management)

### Requirement: Backend SSE envelope parsing
The TUI SHALL parse backend SSE event `data` as `EventEnvelope<SystemEvent>` and dispatch the inner `SystemEvent` into the existing UI update flow.

#### Scenario: Envelope is parsed and dispatched
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an SSE message is received with JSON-encoded `EventEnvelope<SystemEvent>` in `data`
- **THEN** the TUI SHALL parse the envelope
- **AND** it SHALL dispatch the inner `SystemEvent` for handling

### Requirement: Reasoning summary visibility toggle
The TUI SHALL allow toggling the visibility of reasoning summaries.

#### Scenario: Summaries are shown by default
- **GIVEN** the TUI starts in a new session
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_summary"` is received
- **THEN** the TUI MUST display it as a system transcript entry

#### Scenario: Toggle hides reasoning summaries
- **GIVEN** the user disables reasoning summary visibility
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_summary"` is received
- **THEN** the TUI MUST hide it

#### Scenario: /think toggles summary visibility
- **GIVEN** the TUI is running
- **WHEN** the user executes `/think`
- **THEN** the TUI SHALL toggle reasoning summary visibility

### Requirement: Debug-only raw reasoning display
If raw reasoning content is ever made available to the TUI, it MUST only be displayable when debug mode is enabled.

#### Scenario: Debug gate is enforced
- **GIVEN** debug mode is disabled
- **WHEN** a `MessageReceived` system event with `kind = "reasoning_raw"` is received
- **THEN** the TUI MUST NOT render raw reasoning content

