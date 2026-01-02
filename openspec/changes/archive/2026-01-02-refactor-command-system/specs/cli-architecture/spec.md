# cli-architecture Specification

## ADDED Requirements

### Requirement: REPL Session Switching
The CLI REPL SHALL adopt new session IDs returned by the server after lifecycle commands.

#### Scenario: REPL Updates Session ID After /new
Given the CLI is connected to a server using session id `S1`
When the user sends "/new"
And the server returns a chat response indicating a new session id `S2`
Then the CLI SHALL store `S2` as the active session id
And subsequent chat requests SHALL use `S2`
