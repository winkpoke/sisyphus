## ADDED Requirements

### Requirement: CLI Headless Server Management
The CLI SHALL manage a local server instance when no remote server is specified.
#### Scenario: CLI Spawns Server on Demand
- **Given** the user runs `sisyphus` without arguments
- **When** the command starts
- **Then** a new `sisyphus serve` process is spawned
- **And** the server uses the port defined in the configuration file or the default code-defined port
- **And** the CLI connects to this local server
- **And** the server is terminated when the CLI exits

### Requirement: CLI Remote Connection
The CLI SHALL support connecting to an external server instance via a URL argument.
#### Scenario: CLI Connects to Existing Server
- **Given** a running sisyphus server on port 4000
- **When** the user runs `sisyphus attach http://localhost:4000`
- **Then** the CLI connects to the existing server
- **And** no new server process is spawned
- **And** the server remains running when the CLI exits

### Requirement: Event Streaming
The CLI SHALL consume real-time events from the server to display agent activity.
#### Scenario: Real-time Event Streaming
- **Given** a connected CLI session
- **When** the user sends a message
- **Then** the CLI receives updates (thoughts, text chunks) via SSE
- **And** renders them progressively to the terminal
