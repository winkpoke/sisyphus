## MODIFIED Requirements

### Requirement: Start a new session via command
The system SHALL clear the current session context when a new session command is executed.

The command-driven session lifecycle transition MUST be applied by a single core chat-handling entrypoint so that:
- Session context is cleared exactly once
- The effective session id returned to clients is deterministic
- No HTTP-layer handler duplicates effect application

For this change, “session context” is defined as:
- The conversation history used to build completion requests (user and assistant messages)
- The tool-result history used to build completion requests

#### Scenario: New session command clears context
- **GIVEN** an existing session with prior context
- **WHEN** the user executes the new session slash command
- **THEN** the session context MUST be cleared before the next completion request

Clearing context MUST be externally observable:
- The next completion request MUST NOT include any prior messages from the previous session
- The next completion request MUST NOT include any prior tool results from the previous session

### Requirement: Clear history via command
The system SHALL clear the current session context when a clear-history command is executed.

The clear-history effect MUST be applied by a single core chat-handling entrypoint so that:
- Session context is cleared exactly once
- No HTTP-layer handler duplicates effect application

#### Scenario: Clear history command clears context
- **GIVEN** an existing session with prior context
- **WHEN** the user executes the clear-history slash command
- **THEN** the session context MUST be cleared before the next completion request

Clearing context MUST be externally observable:
- The next completion request MUST NOT include any prior messages
- The next completion request MUST NOT include any prior tool results

