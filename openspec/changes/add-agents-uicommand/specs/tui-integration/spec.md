# TUI Architecture Spec Delta

## ADDED Requirements

#### Scenario: User lists available agents in TUI
- **Given** the user is in the TUI
- **When** the user runs the `/agents` command
- **Then** the application fetches the list of agents from the server
- **And** displays an interactive selection list (modal/overlay)
- **And** the list includes agent names, IDs, and descriptions
- **And** the current session's agent is visually indicated

#### Scenario: User selects an agent in TUI
- **Given** the agent selection list is open
- **When** the user navigates to an agent and presses Enter
- **Then** the application sends a request to update the session's agent
- **And** closes the selection list
- **And** shows a success notification (toast) upon completion

#### Scenario: User cancels agent selection
- **Given** the agent selection list is open
- **When** the user presses Esc
- **Then** the selection list closes without making changes

#### Scenario: Error handling during agent operations
- **Given** the user triggers an agent operation (list or switch)
- **When** the server request fails
- **Then** an error notification (toast) is displayed
