# llm-provider Specification

## Purpose
TBD - created by archiving change scaffold-phase-1. Update Purpose after archive.
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

