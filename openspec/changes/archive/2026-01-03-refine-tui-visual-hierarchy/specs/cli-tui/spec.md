# CLI TUI Visual Improvements

## ADDED Requirements

### Status Bar
The application MUST display a structured status bar at the bottom of the screen.

#### Scenario: Status Bar Components
Given the application is running
Then the status bar should show the Session ID on the left
And the Active Model and Token Usage in the center
And the System Status (Connected/Disconnected) on the right.

#### Scenario: Processing State
Given the user has sent a message
When the agent is processing
Then the status bar should show an animated spinner
And the status text should indicate "Generating..."

### Requirement: Transcript Visuals
The transcript view MUST provide adequate whitespace and context.

#### Scenario: Dynamic Header
Given a session is active
Then the transcript block title should display the current Context Name or Session ID
Instead of the static text "Transcript".

#### Scenario: Content Padding
Given the transcript is displaying messages
Then there should be padding between the text and the block borders
To improve readability.
