# Change: Add Accurate Token Estimation

## Why

The current token estimation implementation in `DefaultTokenEstimator` uses a simple character-based approximation (`chars / 4 + 4`). This approach has several issues:

1. **Inaccuracy**: Different languages and content types have different token densities. Code with dense operators, Chinese/Japanese text, or technical jargon can vary significantly from `chars / 4`.

2. **Budget violation risk**: Inaccurate estimation means we might think we're within token limits when we're actually over, or we might be too conservative and drop context unnecessarily.

3. **No model-specific accuracy**: Different LLM providers (Anthropic, OpenAI, Google) use different tokenization algorithms. A single approximation cannot accurately estimate tokens for all models.

### OpenCode's Approach

Based on research of OpenCode and similar projects:

**OpenCode uses provider-specific tokenizers via official SDKs**:
- **Anthropic models**: Uses `anthropic_sdk::tokens::TokenCounter` from the official Anthropic Rust SDK for accurate cl100k tokenizer counting
- **OpenAI models**: Uses tiktoken-rs for OpenAI's GPT models (different encodings: p50k_base, cl100k_base, r50k_base, etc.)
- **Other providers**: Falls back to character-based approximation or provider-specific tokenizers when available

**OpenCode's token counting strategy**:
1. **Provider-specific**: Different LLM providers use different tokenization algorithms (cl100k for Anthropic, tiktoken for OpenAI, etc.)
2. **Model-aware**: Uses model name to select appropriate encoding (e.g., `claude-3-opus-20240229` uses cl100k)
3. **Accurate SDK usage**: Leverages official provider SDKs (anthropic_sdk for Claude, tiktoken-rs for OpenAI) rather than re-implementing tokenizers
4. **Performance optimization**: Tokens are cached and reused across context renders to avoid repeated counting

Implementing accurate token estimation using a proper tokenizer library (like `tiktoken-rs`) provides model-specific accuracy, better context management, and prevents unexpected budget violations.

## What Changes

- Add `tiktoken-rs` dependency for accurate token counting
- Refactor `TokenEstimator` trait to support multiple estimation strategies (Exact, Heuristic, Hybrid)
- Implement `TiktokenEstimator` using `tiktoken-rs` for model-specific accuracy
- Update `ContextLimits` configuration to include optional estimation strategy selection
- Update context rendering to use configurable estimator
- Add model name to `Context` metadata for accurate tokenizer selection

## Impact

- **Affected specs**:
  - `session-core` - New requirements for estimation strategies
  - `llm-provider` - May need model name propagation to support tokenizer selection

- **Affected code**:
  - `crates/core/src/session/context.rs` - New `TiktokenEstimator`, updated `TokenEstimator` trait
  - `crates/core/src/agent/config.rs` - Updated `ContextLimits` to include estimation strategy
  - `crates/core/Cargo.toml` - Add `tiktoken-rs` dependency

- **Breaking changes**: None - Default `DefaultTokenEstimator` remains available as fallback
