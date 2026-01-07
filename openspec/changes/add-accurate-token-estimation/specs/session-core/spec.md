## ADDED Requirements

### Requirement: Configurable Token Estimation Strategies

The system SHALL support multiple token estimation strategies to balance accuracy, performance, and compatibility across different LLM providers.

#### Scenario: Select estimation strategy based on provider

- **GIVEN** an Agent configuration specifies a token estimation strategy
- **WHEN** the agent renders context for an LLM provider
- **THEN** the system SHALL use the specified estimation strategy:
  - `Exact`: Use model-specific tokenizer (e.g., tiktoken for OpenAI, cl100k for Anthropic)
  - `Heuristic`: Use language-aware character-based approximation
  - `Hybrid`: Use tokenizer with heuristic fallback

#### Scenario: Fallback to default estimator when strategy unavailable

- **GIVEN** an Agent configuration specifies an estimation strategy that is not implemented
- **WHEN** the agent renders context
- **THEN** the system SHALL fall back to `DefaultTokenEstimator`
- **AND** the system SHALL log a warning indicating the fallback occurred

#### Scenario: Model-specific tokenizer selection

- **GIVEN** a model name is available in the context (e.g., `anthropic/claude-opus-4.5`, `openai/gpt-4`)
- **WHEN** the agent uses `Exact` estimation strategy
- **THEN** the system SHALL select the appropriate tokenizer for that model family:
  - Anthropic models: Use cl100k_base tokenizer
  - OpenAI models: Use tiktoken cl100k_base
  - Google models: Use gemma tokenizer (if available)
  - Other models: Use fallback heuristic estimation

### Requirement: Accurate Token Estimation via Tiktoken

The system SHALL provide accurate token counting using the `tiktoken-rs` library for OpenAI models and model-specific tokenizers for other providers.

#### Scenario: Exact estimation with tiktoken for OpenAI models

- **GIVEN** the estimation strategy is set to `Exact` for an OpenAI model
- **WHEN** `estimate_message_tokens()` is called on a message
- **THEN** the system SHALL use `tiktoken-rs` to encode the message content
- **AND** the system SHALL return the actual token count from the tokenizer
- **AND** the estimation SHALL include special tokens for message framing (role, etc.)

#### Scenario: Token estimation includes tool calls and results

- **GIVEN** a message contains tool calls or tool_call_id fields
- **WHEN** `estimate_message_tokens()` is called
- **THEN** the system SHALL estimate tokens for all message components:
  - Message content
  - Tool call IDs, kinds, function names, and arguments
  - Tool call IDs
- **AND** the system SHALL return the sum of all component tokens

### Requirement: Token Estimator Trait Extension

The system SHALL extend the `TokenEstimator` trait to support multiple estimation strategies.

#### Scenario: Runtime strategy selection

- **GIVEN** a `ContextLimits` configuration specifies an estimation strategy
- **WHEN** a `TokenEstimator` is instantiated based on that strategy
- **THEN** the system SHALL create the appropriate estimator instance:
  - `TiktokenEstimator` for `Exact` strategy with OpenAI models
  - `HeuristicEstimator` for `Heuristic` strategy
  - `DefaultTokenEstimator` for fallback or compatibility

### Requirement: Context Limits Configuration Extension

The system SHALL support configuring estimation strategy in `ContextLimits`.

#### Scenario: Configure estimation strategy via agent config

- **GIVEN** an Agent configuration includes `context_limits`
- **WHEN** the configuration is loaded
- **THEN** the `context_limits` SHALL include an optional `estimation_strategy` field:
  ```rust
  pub struct ContextLimits {
      pub max_prompt_tokens: u32,
      pub reserved_completion_tokens: Option<u32>,
      pub estimation_strategy: Option<EstimationStrategy>,
  }

  pub enum EstimationStrategy {
      Exact,
      Heuristic,
      Hybrid,
  }
  ```
- **AND** the system SHALL use this strategy when rendering context if specified

### Requirement: Backward Compatibility with Default Estimator

The system SHALL maintain backward compatibility by keeping `DefaultTokenEstimator` available as a fallback.

#### Scenario: Legacy usage without explicit strategy

- **GIVEN** an existing Agent configuration does not specify an estimation strategy
- **WHEN** context is rendered
- **THEN** the system SHALL use `DefaultTokenEstimator`
- **AND** the system SHALL NOT require changes to existing configurations

#### Scenario: Explicit opt-in for new estimation

- **GIVEN** a project wants to use accurate token estimation
- **WHEN** they add `estimation_strategy: EstimationStrategy::Exact` to their `ContextLimits`
- **THEN** the system SHALL use the accurate estimator based on that strategy
