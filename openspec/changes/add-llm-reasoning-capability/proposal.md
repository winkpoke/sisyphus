# Change: Add LLM reasoning capability (config + provider support)

## Why
Some OpenAI-compatible routers and models provide a “reasoning/thinking” mode that improves tool planning and answer quality, but Sisyphus cannot enable or safely surface it today.

## What Changes
- Add optional reasoning configuration (mode/effort/exposure/storage).
- Add provider-agnostic request payload overrides for OpenAI-compatible endpoints.
- Allow providers to map reasoning outputs into a safe “reasoning summary” channel.
- Keep raw chain-of-thought hidden by default; allow debug-only display with redaction + truncation.

## Impact
- Affected specs: `llm-provider`, `agent-core`, `cli-tui`
- Affected code:
  - `crates/common/src/config.rs` (config schema)
  - `crates/common/src/llm.rs` (request/response model)
  - `crates/provider/src/openai.rs` (payload merge + response mapping)
  - `crates/core/src/agent.rs` (request options + event emission)
  - `crates/cli/src/ui/tui/*` (toggle + rendering policy)
- Security:
  - Do not persist raw reasoning by default.
  - Redact + truncate any debug-visible raw reasoning.

