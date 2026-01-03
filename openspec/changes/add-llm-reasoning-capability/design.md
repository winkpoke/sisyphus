# Design: Reasoning capability for OpenAI-compatible providers

## Overview
Add an opt-in “reasoning” capability that can be enabled per configuration and applied to OpenAI-compatible API requests. The system may surface a safe reasoning summary while keeping raw chain-of-thought hidden by default.

This design intentionally supports OpenAI-compatible routers (e.g., custom `base_url`) without requiring a new provider implementation per vendor.

## Key Concepts

### Reasoning request controls
Introduce a normalized reasoning request object (conceptual):
- `mode`: `off | on | auto`
- `effort`: `low | medium | high` (provider-dependent mapping)
- `expose`: `none | summary | debug`
- `store`: `none | summary`

`auto` means “enable reasoning only when tool calls are available or when the user explicitly requests it”, to keep cost/latency predictable.

### Provider-agnostic request overrides
Support a JSON object (`request_overrides`) that is deep-merged into the provider payload after Sisyphus populates required fields (`model`, `messages`, `tools`, etc.).

Merge precedence (lowest to highest):
1. Provider defaults
2. Sisyphus standard fields
3. Sisyphus normalized reasoning mapping
4. `request_overrides`

This allows vendor-specific keys (e.g., `reasoning_effort`, `thinking`, `enable_reasoning`) without hardcoding them.

### Reasoning outputs
Providers vary:
- Some return reasoning in a separate field (e.g., `reasoning_content`, `thoughts`).
- Some embed “thinking” in the main content.

Sisyphus will map provider-specific reasoning output into:
- `reasoning_summary`: safe to display and optionally store
- `reasoning_raw`: debug-only, never stored by default

### Storage and exposure policy
Default policy:
- Do not store raw reasoning in session context.
- Do not display raw reasoning.

When `expose=summary`:
- Display only `reasoning_summary` as a system transcript entry.

When `expose=debug`:
- Display raw reasoning only when UI debug mode is enabled.
- All raw reasoning display must be redacted + truncated.

## Event propagation
To avoid breaking existing event consumers, reasoning summaries SHOULD be sent as an additional `SystemEvent::MessageReceived` with `role = "system"` and a stable prefix.

Rationale:
- The server currently broadcasts `SystemEvent` objects as JSON over SSE without versioning.
- Adding new enum variants is feasible, but increases client coupling.

## Security notes
- Never treat reasoning text as instruction authority for tool execution.
- Never include raw reasoning in tool arguments.
- Redact sensitive material in any debug-visible reasoning payloads.

