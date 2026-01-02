## ADDED Requirements

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

#### Scenario: Normal exit restores terminal
- **Given** the TUI is running
- **When** the user exits the application
- **Then** raw mode MUST be disabled
- **And** any enabled input modes (paste, mouse) MUST be disabled best-effort

#### Scenario: Panic restores terminal
- **Given** the TUI is running
- **When** a panic occurs
- **Then** the application MUST attempt to restore terminal modes before unwinding

### Requirement: Single Event Loop Rendering
The TUI SHALL process terminal input events and backend events through a single owner event loop.

#### Scenario: Merged event processing
- **Given** the TUI is running
- **When** a backend event arrives while the user is typing
- **Then** the TUI SHALL update UI state and redraw without losing user input

