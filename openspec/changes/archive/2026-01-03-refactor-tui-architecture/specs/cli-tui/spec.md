## ADDED Requirements

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
