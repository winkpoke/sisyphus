# agent-core Specification (Delta)

## MODIFIED Requirements

### Requirement: Agent Loop Testing
The system SHALL provide comprehensive unit and integration tests for the agent loop to verify decision logic, tool execution flow, and error recovery.

#### Scenario: Single-turn chat without tools
- **GIVEN** an agent with mock LLM provider
- **WHEN** chat is called with simple message
- **THEN** agent calls LLM provider once
- **AND** session history contains user and assistant messages
- **AND** no tools are executed

#### Scenario: Multi-turn conversation
- **GIVEN** an agent with mock LLM provider
- **WHEN** chat is called multiple times
- **THEN** agent maintains conversation context
- **AND** session history accumulates all messages
- **AND** context compaction triggers when token limit reached

#### Scenario: Tool execution flow
- **GIVEN** an agent with mock LLM returning tool call
- **AND** mock tool registered
- **WHEN** chat is called with message requesting tool usage
- **THEN** agent executes tool
- **AND** tool result is appended to session
- **AND** agent makes second LLM call with tool result

#### Scenario: Sequential tool calls in single message
- **GIVEN** an agent with mock LLM returning multiple tool calls
- **AND** multiple mock tools registered
- **WHEN** chat is called
- **THEN** agent executes tools in order
- **AND** all tool results are appended to session
- **AND** agent makes final LLM call with all tool results

#### Scenario: Provider error handling
- **GIVEN** an agent with mock LLM provider configured to fail
- **WHEN** chat is called
- **THEN** agent handles error gracefully
- **AND** error is returned to caller
- **AND** session state is not corrupted
- **AND** error is logged appropriately

#### Scenario: Tool execution failure handling
- **GIVEN** an agent with mock LLM returning tool call
- **AND** mock tool configured to return error
- **WHEN** chat is called
- **THEN** agent handles tool error
- **AND** error message is included in context
- **AND** agent continues with error context

#### Scenario: Max iterations limit enforcement
- **GIVEN** an agent with max_iterations configured
- **AND** mock LLM returning tool calls
- **WHEN** chat is called requiring more than max_iterations
- **THEN** agent stops after max_iterations
- **AND** appropriate error is returned
- **AND** session history is not corrupted

#### Scenario: Context window management
- **GIVEN** an agent with limited context window
- **AND** mock LLM returning responses
- **WHEN** chat is called repeatedly to fill context
- **THEN** agent respects context limit
- **AND** context compaction is triggered
- **AND** older messages are removed or summarized
- **AND** recent messages are preserved

#### Scenario: Empty user message handling
- **GIVEN** an agent with mock LLM provider
- **WHEN** chat is called with empty message
- **THEN** agent handles gracefully
- **AND** either returns error or minimal response
- **AND** session is not modified unnecessarily

#### Scenario: Concurrent chat requests
- **GIVEN** an agent with shared state
- **WHEN** multiple simultaneous chat requests are made to same session
- **THEN** agent returns error indicating session is busy
- **AND** error message is descriptive
- **AND** no data races occur
- **AND** session state is not corrupted

#### Scenario: Concurrent chat to different sessions
- **GIVEN** an agent managing multiple sessions
- **WHEN** simultaneous chat requests are made to different sessions
- **THEN** both requests are processed concurrently
- **AND** sessions are isolated from each other
- **AND** no cross-session contamination occurs

## ADDED Requirements

### Requirement: Agent Unit Test Organization
The system SHALL organize agent tests into `#[cfg(test)]` modules for fast execution and integration tests in `tests/` directory for end-to-end flows.

#### Scenario: Unit tests use mocks
- **GIVEN** agent unit tests in `src/agent.rs`
- **WHEN** tests are run
- **THEN** all tests use mock LLM providers
- **AND** all tests use mock tools
- **AND** no external dependencies are required

#### Scenario: Integration tests use real components
- **GIVEN** agent integration tests in `tests/`
- **WHEN** tests are run
- **THEN** tests verify agent loop with real session
- **AND** tests use scripted/mock providers
- **AND** tests verify permission flows

### Requirement: Mock Framework Adoption
The system SHALL use `mockall` framework for creating mock LLM providers and tools in agent tests.

#### Scenario: Mock LLM provider
- **GIVEN** an agent test requiring mock provider
- **WHEN** test sets up `MockLLMProvider::new()`
- **THEN** mockall creates a valid mock object
- **AND** expectations can be set on the mock
- **AND** mock returns scripted responses

#### Scenario: Mock tool expectations
- **GIVEN** an agent test requiring mock tool
- **WHEN** test configures mock tool expectations
- **THEN** mockall verifies expected calls
- **AND** mockall verifies call arguments
- **AND** mockall returns specified results

### Requirement: Snapshot Testing for Prompts
The system SHALL use `insta` for snapshot testing of generated system prompts to detect unintended changes.

#### Scenario: System prompt snapshot
- **GIVEN** an agent configuration with instructions
- **WHEN** system prompt is generated
- **THEN** snapshot matches expected output
- **AND** XML tags are correctly formatted
- **AND** environment variables are included

#### Scenario: Template prompt snapshot
- **GIVEN** an agent configuration with custom template
- **WHEN** system prompt is generated from template
- **THEN** snapshot matches expected output
- **AND** template variables are interpolated correctly
- **AND** conditional sections work as expected
