# cli-tui Specification Delta

## ADDED Requirements

### Requirement: Immersive Full-Screen Layout
The TUI SHALL utilize the entire available terminal screen space without outer margins.

#### Scenario: Full-screen utilization
- **Given** the TUI is running
- **Then** the main application layout MUST NOT have external margins
- **And** the interface elements SHALL extend to the edges of the terminal window

### Requirement: Persistent Context Header
The TUI SHALL display a persistent header bar at the top of the screen containing session context.

#### Scenario: Context Header Content
- **Given** the TUI is active
- **Then** the top row SHALL display a Context Header
- **And** the header MUST include the application name (Sisyphus)
- **And** the header MUST include the current working directory
- **And** the header MUST include the active model name

#### Scenario: Long working directory truncation
- **Given** the working directory path exceeds available header width
- **When** the ContextBar is rendered
- **Then** the directory SHALL be truncated with ellipsis in the middle (e.g., `/Users/.../sisyphus`)
- **And** the most meaningful parts of the path SHALL be preserved

#### Scenario: Model name unavailable
- **Given** the active model is "Loading..." or empty
- **When** the ContextBar is rendered
- **Then** the model section SHALL display a fallback indicator ("Loading..." or "N/A")

#### Scenario: Context header colorblind accessibility
- **Given** the TUI is active with colorblind accessibility mode
- **When** the ContextBar is rendered
- **Then** the header SHALL include an underline or distinct border pattern
- **And** the pattern SHALL differentiate from other UI elements independently of color

### Requirement: Current Working Directory Tracking
The TUI SHALL track and display the current working directory in the Context Header.

#### Scenario: Directory tracking initialization
- **Given** a new session starts
- **When** the TUI initializes
- **Then** `current_working_directory` SHALL be set to the actual working directory
- **And** it SHALL be displayed in the Context Header

#### Scenario: Directory change tracking
- **Given** the TUI is running
- **When** a session event indicates a directory change
- **Then** the `current_working_directory` field SHALL be updated
- **And** the Context Header SHALL reflect the new path immediately

### Requirement: Minimalist Input Composer
The input area SHALL be a minimalist, prompt-driven component without heavy borders.

#### Scenario: Prompt Symbol
- **Given** the input area is visible
- **Then** it SHALL display a prompt symbol (e.g., `> `) at the start of the input line
- **And** it SHALL NOT be enclosed in a block with the title "Input"

#### Scenario: Focus Indication
- **Given** the user is typing in the input composer
- **Then** the prompt character SHALL be highlighted using the `highlight` theme color
- **And** the prompt background SHALL use the `context_bar_bg` theme color

#### Scenario: Inactive Input Styling
- **Given** the user is not typing (e.g., streaming response)
- **Then** the prompt character SHALL be rendered using the `system` theme color
- **And** the input area SHALL be visually distinguishable from the active state

#### Scenario: Multi-line Input
- **Given** the user types multiple lines in the input composer
- **When** the content exceeds one line
- **Then** the input area SHALL expand vertically to fit content
- **And** expansion SHALL be limited to maximum 3 lines
- **And** scrolling SHALL be enabled if content exceeds 3 lines

### Requirement: Minimum Terminal Size Support
The TUI SHALL define and support a minimum terminal size requirement.

#### Scenario: Minimum Size Requirement
- **Given** the TUI is initialized
- **When** the terminal size is checked
- **Then** the minimum size SHALL be 24 rows × 80 columns
- **And** the TUI SHALL render successfully at this size

#### Scenario: Degraded Mode for Small Terminals
- **Given** the terminal size is below 24×80 columns
- **When** the TUI attempts to render
- **Then** it SHALL display a warning banner indicating degraded mode
- **And** it SHALL fallback to prefix-based message rendering (no MessageBlock headers)
- **And** it SHALL preserve basic functionality

#### Scenario: Terminal Resize Handling
- **Given** the TUI is running
- **When** the terminal is resized above or below the minimum size
- **Then** the TUI SHALL automatically switch between normal and degraded mode
- **And** the ContextBar truncation SHALL adjust to the new width

### Requirement: Streaming Performance Optimization
The TUI SHALL maintain high rendering performance during message streaming with the MessageBlock layout.

#### Scenario: Partial Message Re-render
- **Given** the assistant is streaming a response
- **When** a new chunk arrives
- **Then** the TUI SHALL re-render only the modified message block
- **And** it SHALL NOT re-render the entire transcript

#### Scenario: Scroll Update Batching
- **Given** the assistant is streaming a long response
- **When** multiple scroll updates occur within a frame
- **Then** scroll updates SHALL be batched to a maximum of 60 FPS
- **And** excess updates SHALL be coalesced

#### Scenario: Layout Caching
- **Given** the assistant is streaming a response
- **When** message content grows
- **Then** layout calculations SHALL be cached between chunks
- **And** recalculations SHALL be minimized

## MODIFIED Requirements

### Requirement: Transcript Visuals
The transcript view MUST render messages as distinct visual blocks with styled headers and SHALL NOT use the previous block-bordered layout that provided adequate whitespace and context.

#### Scenario: Message Block Rendering
- **Given** the transcript contains messages
- **When** the TUI renders a message
- **Then** it SHALL display a distinct header line indicating the sender (User/Assistant/System)
- **And** the header SHALL use semantic coloring (Blue for User, Lavender for Assistant, Grey for System)
- **And** the header SHALL include a border pattern (solid vs dashed) for colorblind accessibility
- **And** the message content SHALL be rendered below the header without borders
- **And** there SHALL be a 1-blank-line separator between distinct messages

#### Scenario: Message Block Streaming Indicator
- **Given** the assistant is generating a response
- **When** the TUI renders the incomplete message block
- **Then** a blinking cursor or spinner SHALL be displayed at the end of the last line
- **And** the indicator SHALL be removed once the message generation is complete

#### Scenario: Message Block Scroll Calculation
- **Given** the transcript contains multiple message blocks
- **When** scroll offset is calculated
- **Then** the calculation MUST include header height (1 line) + separator (1 line) per message
- **And** scroll position SHALL remain accurate with message block layout

#### Scenario: Message Block Colorblind Accessibility
- **Given** the TUI is active with colorblind accessibility mode
- **When** message blocks are rendered
- **Then** User message headers SHALL use a solid border pattern
- **And** Assistant message headers SHALL use a dashed border pattern
- **And** System message headers SHALL use a dotted border pattern
- **And** the patterns SHALL be distinguishable independently of color

### Requirement: Status Indicators
The status indicators SHALL be integrated into a unified footer in the same line as the input area and MUST NOT occupy a separate status bar section, while providing unobtrusive status indicators and key hints for discoverability.

#### Scenario: Unified Status Footer Layout
- **Given** the TUI is active
- **When** the unified footer is rendered
- **Then** the footer SHALL span the full width of the terminal
- **And** it SHALL be split horizontally: Input area (70%) on the left, Status area (30%) on the right
- **And** it SHALL occupy minimal vertical space (1-3 lines depending on input length)

#### Scenario: Status Content Format
- **Given** the unified footer is rendered
- **Then** the status area SHALL display connection status and token usage
- **And** the format SHALL be `[Status] [Tokens]` (e.g., `[● Connected] [1234/4096]`)
- **And** it SHALL use existing theme colors (`success` for connected, `error` for disconnected, `highlight` for processing)

#### Scenario: Session ID Location
- **Given** the TUI is active
- **Then** the Session ID SHALL be displayed in either the Context Header (right side) or Status Footer
- **And** the location SHALL be consistent with the design specification

## REMOVED Requirements

### Requirement: Dynamic Header
This requirement is removed because the persistent ContextBar now provides session context.

### Requirement: Content Padding
This requirement is removed because MessageBlock headers provide visual separation instead of block borders.
