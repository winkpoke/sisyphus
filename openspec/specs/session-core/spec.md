# session-core Specification

## Purpose
Defines session management architecture including conversation history storage, context compaction with token budgeting, turn-based message tracking, and pending approval state. Sessions track associated agents and provide concurrent-safe access via granular locking.
## Requirements
### Requirement: Session History
The system SHALL store the linear history of conversation messages (User, Assistant, Tool) within the Session object via a dedicated context component.

The system prompt is generated dynamically by the Agent and injected at completion request time; it is not required to be stored in the session context.

#### Scenario: Append message
- **WHEN** a message is added to the session
- **THEN** it is appended to the session context
- **AND** it is available for the next context window generation

#### Scenario: Preserve multi-step turn ordering
- **GIVEN** a single user input that triggers multiple assistant/tool steps
- **WHEN** those steps are appended to the session
- **THEN** the session context preserves their order deterministically
- **AND** the context window generation reflects the same order

### Requirement: Stateless Agent Execution
The Agent SHALL NOT maintain internal session state.

#### Scenario: Execute Turn
- **WHEN** the Agent executes a chat turn
- **THEN** it accepts a `Session` reference as an input
- **AND** reads history from that Session
- **AND** writes new messages to that Session

### Requirement: Context Window Construction
The system SHALL construct a provider-ready context window from the session context for each LLM completion request.

#### Scenario: Render context window
- **GIVEN** a session context containing prior messages
- **WHEN** the system builds a completion request
- **THEN** the system MUST produce an ordered list of messages suitable for the LLM provider

#### Scenario: Inject system messages at request time
- **GIVEN** a session context containing prior conversation messages
- **WHEN** the system builds a completion request
- **THEN** system messages (such as the dynamic system prompt) are prepended for that request
- **AND** they do not need to be stored in the session context

### Requirement: Safe Context Compaction
The system SHALL support compacting the session context to fit within a configurable prompt token budget.

#### Scenario: Compact by dropping oldest blocks
- **GIVEN** a session context that exceeds the prompt token budget
- **WHEN** the system compacts the context
- **THEN** the system MUST remove the oldest non-pinned context blocks until within budget
- **AND** pinned context blocks MUST be preserved

#### Scenario: Preserve tool exchange atomicity
- **GIVEN** a session context containing an Assistant message with tool calls and the corresponding Tool result messages
- **WHEN** the system compacts the context
- **THEN** it MUST NOT remove only part of the tool exchange

#### Scenario: Fail deterministically when pinned content exceeds budget
- **GIVEN** pinned content and injected system messages exceed the prompt token budget
- **WHEN** the system attempts to build a completion request
- **THEN** it MUST return a deterministic error describing the budget violation

### Requirement: Granular Session Locking
The system SHALL support granular locking for sessions to prevent global blocking during long-running operations.

#### Scenario: Concurrent Session Access
Given a running server with multiple active sessions
When one user initiates a chat request that triggers a long-running LLM inference
Then other users should still be able to create new sessions
And other users should still be able to list sessions
And other users should still be able to chat in their own sessions

### Requirement: Session Isolation
The system SHALL isolate session locks so that accessing one session does not block others.

#### Scenario: Session Isolation
Given a `SessionManager` using `DashMap`
When a session is accessed via `get_session`
Then the returned reference should be an `Arc<RwLock<Session>>`
And locking this session should not block access to other sessions in the map

### Requirement: Session Tracks Current Agent
The Session entity SHALL track the identity of the Agent responsible for executing its chat turns.

#### Scenario: Session Stores Agent ID
- **WHEN** a new session is created by the SessionManager
- **THEN** the Session MUST store an `agent_id` field representing the associated Agent
- **AND** this field MUST be included in the Session's serialized representation used by the server API.

#### Scenario: Agent ID is Stable During Turn
- **GIVEN** an existing session `S1` with `agent_id` `A1`
- **WHEN** the Agent begins processing a chat turn for `S1`
- **THEN** the `agent_id` used for that turn MUST remain `A1` for the duration of the turn
- **AND** any concurrent request to change `S1`'s `agent_id` MUST be rejected according to the server API.

### Requirement: Session context mutations are not driven by SlashCommands
The system SHALL NOT require SlashCommands to mutate session lifecycle state.

#### Scenario: SlashCommand expansion does not clear context
- **GIVEN** a session with prior context
- **WHEN** the user executes a SlashCommand that expands to prompt text
- **THEN** the session context MUST remain intact unless an explicit session operation is invoked

### Requirement: Session Lifecycle Management
The system SHALL maintain the state of a conversation independently of the agent execution logic.

#### Scenario: Create new session
- **WHEN** a new session is requested via SessionManager
- **THEN** a new Session object is created with a unique ID
- **AND** the session status is initialized to "Idle"
- **AND** the session context is empty

#### Scenario: Session Locking
- **WHEN** an agent begins processing a turn for a session
- **THEN** the session status transitions to "Busy"
- **WHEN** the agent finishes processing
- **THEN** the session status transitions back to "Idle"

#### Scenario: Prevent concurrent access
- **WHEN** a request is made to process a session that is "Busy"
- **THEN** the system throws/returns a `SessionBusy` error

#### Scenario: Start a new session via API
- **GIVEN** an existing session with prior context
- **WHEN** a "new session" operation is executed
- **THEN** the session context is cleared
- **AND** the session status remains consistent with the operation lifecycle

#### Scenario: Clear operation preserves lifecycle correctness
- **GIVEN** an existing session with prior context
- **WHEN** the session context is cleared via an explicit clear operation
- **THEN** the session context is cleared before the next completion request
- **AND** the session status remains consistent with the operation lifecycle

### Requirement: Start a new session via API
The system SHALL support starting a new session via an explicit session-creation API.

Session creation MUST produce a new session id and an empty session context.

#### Scenario: Create new session produces empty context
- **GIVEN** an existing session `S1` with prior context
- **WHEN** a new session is created via the session manager
- **THEN** the new session `S2` MUST have an empty message history
- **AND** the new session `S2` MUST have an empty tool-result history

### Requirement: Clear history via API
The system SHALL support clearing an existing session context via a dedicated clear operation.

#### Scenario: Clear operation clears context
- **GIVEN** an existing session with prior context
- **WHEN** the clear operation is executed for that session
- **THEN** the session context MUST be cleared before the next completion request

