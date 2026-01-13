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

