## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Terminal Mode Safety
The TUI MUST restore the terminal to a usable state on normal exit and on panic.

#### Scenario: Overlay usage remains safe
- **Given** the TUI is running with overlays active
- **When** the user exits
- **Then** the terminal MUST be restored to a usable state
