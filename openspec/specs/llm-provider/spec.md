# llm-provider Specification

## Purpose
Defines the LLM provider abstraction layer that unifies multiple LLM backends (OpenAI, Anthropic, Google, local) behind a common trait. Includes robust Server-Sent Events (SSE) parsing for streaming responses with proper UTF-8 and split-chunk handling.
## Requirements
### Requirement: LLMProvider Trait
A unified `LLMProvider` trait MUST be defined to abstract different LLM backends.

#### Scenario: Trait Methods
Given the `LLMProvider` trait
Then it must define `complete` and `stream` methods
And it must be object-safe (usable as `Box<dyn LLMProvider>`).

### Requirement: OpenAI Implementation
An `OpenAIProvider` MUST be implemented matching the `LLMProvider` trait.

#### Scenario: OpenAI Connection
Given a valid `OPENAI_API_KEY`
When `complete` is called on `OpenAIProvider`
Then it sends a request to the OpenAI API
And returns the response content.

### Requirement: Data Model Standardization
Internal data models (`Message`, `Role`) MUST be defined in `common` and mapped to provider-specific formats.

#### Scenario: Message Mapping
Given a `Message` with role `User` and content "Hello"
When it is passed to `OpenAIProvider`
Then it is converted to the correct JSON structure for the OpenAI API.

### Requirement: Robust SSE Parsing Strategy
The system SHALL provide a shared SSE parser capability that ensures data integrity and security independent of the specific LLM provider.

#### Scenario: Split UTF-8 Characters
Given a network stream where a multi-byte character (e.g., `👋` 4 bytes) is split across two chunks (2 bytes + 2 bytes)
When the parser processes these chunks
Then it SHALL buffer the raw bytes
And it SHALL NOT attempt to decode UTF-8 until a complete line is extracted
And the final output SHALL contain the correctly decoded character.

#### Scenario: Maximum Line Length Enforcement
Given an incoming stream that sends a continuous sequence of bytes exceeding 1MB without a newline character
When the parser consumes this stream
Then it SHALL abort the stream processing
And it SHALL return a specific error indicating the buffer limit was exceeded.

#### Scenario: Standard SSE Field Handling
Given an SSE stream containing comments (`: keep-alive`), specific events (`event: update`), and identifiers (`id: 1`)
When the parser processes the stream
Then it SHALL ignore comment lines entirely
And it SHALL parse `event`, `id`, and `data` fields correctly
And it SHALL dispatch a complete event only upon encountering a double newline.

### Requirement: OpenAI Streaming Implementation
`OpenAIProvider::stream` SHALL use the shared SSE parser to consume responses.

#### Scenario: Data Reassembly
Given the OpenAI API returns a JSON payload split across multiple SSE `data:` lines or network chunks
When `OpenAIProvider` consumes the stream
Then it SHALL use the SSE parser to reconstruct the full JSON string for each event
And it SHALL emit `delta.content` only after successfully parsing the JSON.

#### Scenario: [DONE] Handling
Given the OpenAI API sends `data: [DONE]`
When the provider processes this event
Then it SHALL terminate the stream cleanly
And it SHALL NOT emit any further items.

### Requirement: Provider HTTP Mocking
The system SHALL use `wiremock` to test HTTP provider implementations (OpenAI, Anthropic) without calling live APIs.

#### Scenario: OpenAI completion request mocking
- **GIVEN** a wiremock server configured
- **WHEN** OpenAI provider sends completion request
- **THEN** request contains correct JSON body
- **AND** request includes correct headers (Authorization, Content-Type)
- **AND** request matches OpenAI API specification

#### Scenario: OpenAI stream request mocking
- **GIVEN** a wiremock server configured
- **WHEN** OpenAI provider sends stream request
- **THEN** request includes correct headers for SSE
- **AND** server responds with SSE chunks
- **AND** chunks are parsed correctly

#### Scenario: OpenAI response parsing (text)
- **GIVEN** a wiremock server returning text response
- **WHEN** provider completes request
- **THEN** response is parsed to `Message` with role=Assistant
- **AND** content is correctly extracted
- **AND** tool_calls field is None

#### Scenario: OpenAI response parsing (tool calls)
- **GIVEN** a wiremock server returning tool call response
- **WHEN** provider completes request
- **THEN** response is parsed to `Message` with tool_calls populated
- **AND** tool calls include correct structure
- **AND** function arguments are correctly deserialized

#### Scenario: Network error handling
- **GIVEN** a wiremock server configured to timeout
- **WHEN** provider sends request
- **THEN** error is returned appropriately
- **AND** error indicates network timeout
- **AND** no retry logic fails unexpectedly

#### Scenario: API error 429 handling
- **GIVEN** a wiremock server returning 429 status
- **WHEN** provider sends request
- **THEN** error is returned with rate limit indication
- **AND** error message is descriptive
- **AND** no retry storms occur

#### Scenario: API error 500 handling
- **GIVEN** a wiremock server returning 500 status
- **WHEN** provider sends request
- **THEN** error is returned with server error indication
- **AND** no data corruption occurs
- **AND** error is retryable

#### Scenario: Invalid JSON response handling
- **GIVEN** a wiremock server returning malformed JSON
- **WHEN** provider sends request
- **THEN** error is returned indicating parsing failure
- **AND** no invalid state is loaded
- **AND** error message is descriptive

### Requirement: Provider Trait Testing
The system SHALL provide tests for `LLMProvider` trait implementations to ensure all providers conform to the interface.

#### Scenario: Mock provider implements trait
- **GIVEN** a mock provider created with `mockall`
- **WHEN** provider is used in tests
- **THEN** `complete()` method is available
- **AND** `stream()` method is available
- **AND** `model()` method returns correct value

#### Scenario: Provider trait method signatures
- **GIVEN** a provider implementing `LLMProvider` trait
- **WHEN** trait methods are called
- **THEN** all methods accept correct parameter types
- **AND** all methods return correct result types
- **AND** async behavior is consistent

#### Scenario: Provider error types
- **GIVEN** a provider returning errors
- **WHEN** error is encountered
- **THEN** error type is `anyhow::Result`
- **AND** error includes context
- **AND** error is handleable by caller

### Requirement: Snapshot Testing for Request Formatting
The system SHALL use `insta` to snapshot test request formatting for all providers to detect unintended changes.

#### Scenario: OpenAI request snapshot
- **GIVEN** a completion request with messages and tools
- **WHEN** request is formatted for OpenAI
- **THEN** snapshot matches expected JSON structure
- **AND** messages are correctly serialized
- **AND** tool definitions are correctly included

#### Scenario: Function call snapshot
- **GIVEN** a tool call with arguments
- **WHEN** request is formatted
- **THEN** snapshot matches expected function structure
- **AND** arguments are correctly serialized
- **AND** function metadata is correct

#### Scenario: SSE chunk snapshot
- **GIVEN** a streaming response
- **WHEN** SSE chunks are parsed
- **THEN** snapshot matches expected chunk format
- **AND** delta format is correct
- **AND** multi-byte characters are handled

