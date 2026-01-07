## 1. Implementation

- [ ] 1.1 Add `tiktoken-rs` dependency to `Cargo.toml`
- [ ] 1.2 Create `EstimationStrategy` enum in `context.rs`
- [ ] 1.3 Extend `TokenEstimator` trait with strategy factory method
- [ ] 1.4 Implement `TiktokenEstimator` using `tiktoken-rs` for OpenAI models
- [ ] 1.5 Implement model-specific tokenizer mapping (Anthropic cl100k, OpenAI tiktoken, Google gemma)
- [ ] 1.6 Update `ContextLimits` to include `estimation_strategy` field
- [ ] 1.7 Update context rendering to use configurable estimator from limits
- [ ] 1.8 Implement `HeuristicEstimator` with language-aware token density
- [ ] 1.9 Add unit tests for each estimator type
- [ ] 1.10 Update agent config documentation to explain estimation strategies
- [ ] 1.11 Add example configuration showing different strategies
