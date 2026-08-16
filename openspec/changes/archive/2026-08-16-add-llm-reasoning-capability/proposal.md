# Change: Add LLM reasoning capability (config + provider support)

## Why
Some OpenAI-compatible routers and models provide a “reasoning/thinking” mode that improves tool planning and answer quality, but Sisyphus cannot enable or safely surface it today.

## What Changes
- Add reasoning configuration (mode/effort/exposure/storage) with safe defaults.
- Add provider-agnostic request payload overrides for OpenAI-compatible endpoints.
- Add reserved-key protections so overrides cannot change core request fields.
- Allow providers to map reasoning outputs into a safe “reasoning summary” channel.
- Keep raw chain-of-thought hidden by default; allow debug-only display with redaction + truncation.
- Emit reasoning summaries via structured event metadata (no sentinel content prefixes).
- Add a `/think` UiCommand to toggle reasoning summary visibility in interactive UIs (TUI + REPL).
- Provide a default-on behavior with a safe fallback when providers do not support summaries.

## Priorities and ROI
- P0 (high ROI): Support default-on reasoning summary visibility without exposing raw chain-of-thought.
- P0 (high ROI): Add override guardrails to prevent bypassing required fields and tool control.
- P0 (high ROI): Add `/think` to make summary visibility discoverable and fast to disable.
- P1 (medium ROI): Add `auto` reasoning mode with deterministic, testable enablement rules.
- P2 (lower ROI): Extend provider-specific raw reasoning surfacing beyond OpenAI-compatible paths.

## Impact
- Affected specs: `llm-provider`, `agent-core`, `cli-tui`, `slash-commands`
- Affected code:
  - `crates/common/src/config.rs` (config schema)
  - `crates/common/src/llm.rs` (request/response model)
  - `crates/provider/src/openai.rs` (payload merge + response mapping)
  - `crates/core/src/agent.rs` (request options + event emission)
  - `crates/cli-core/src/ui/repl.rs` (local UiCommand routing + rendering policy)
  - `crates/tui/*` (toggle + rendering policy)
- Security:
  - Do not persist raw reasoning by default.
  - Redact + truncate any debug-visible raw reasoning.
