## MODIFIED Requirements

### Requirement: Event Streaming
The CLI SHALL consume real-time events from the server to display agent activity.

#### Scenario: Real-time Event Streaming
- **Given** a connected CLI session
- **When** the user sends a message
- **Then** the CLI receives updates (thoughts, text chunks) via SSE
- **And** renders them progressively to the terminal
- **And** progressive rendering MUST remain stable under streaming (no flicker-driven input loss)

