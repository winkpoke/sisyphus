# slash-commands Specification

## Purpose
TBD - created by archiving change implement-slash-commands. Update Purpose after archive.
## Requirements
### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

#### Scenario: Discoverable command listing
- **Given** the user is using the interactive TUI
- **When** the user requests command discovery (e.g., typing `/`)
- **Then** the UI SHALL present available slash commands without requiring the user to memorize names

### Requirement: Command Loading Separation
The system SHALL separate command loading logic from the runtime registry to ensure clean architecture and testability.

#### Scenario: Loading Commands
Given a directory of Markdown command files
When the application starts
Then `CommandLoader` should scan and parse the files
And return a collection of command configurations
And `CommandRegistry` should be populated from this collection
But `CommandRegistry` should not contain any file I/O logic

### Requirement: Command Registry Responsibility
The system SHALL ensure the CommandRegistry is responsible only for storage and retrieval.

#### Scenario: Command Registry Responsibility
Given a `CommandRegistry`
When `register_builtin` or `register_custom` is called
Then it should store the command in memory
And it should allow retrieval by name

