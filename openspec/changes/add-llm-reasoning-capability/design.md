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

`auto` means “enable reasoning only when the agent is in a tool-using phase that is likely to require planning”, to keep cost/latency predictable.

Deterministic enablement rules for `auto`:
- Enable when the session contains at least one Tool message in its stored context, indicating the agent is actively using tools.
- Enable when the session contains a pending approval or pending tool batch, indicating the agent is resuming a tool-mediated turn.
- Otherwise, keep reasoning off.

### Provider-agnostic request overrides
Support a JSON object (`request_overrides`) that is deep-merged into the provider payload after Sisyphus populates required fields (`model`, `messages`, `tools`, etc.).

Merge precedence (lowest to highest):
1. Provider defaults
2. Sisyphus standard fields
3. Sisyphus normalized reasoning mapping
4. `request_overrides`

This allows vendor-specific keys (e.g., `reasoning_effort`, `thinking`, `enable_reasoning`) without hardcoding them.

Override guardrails:
- `request_overrides` MUST NOT be able to override reserved request keys that would change core semantics (e.g., `model`, `messages`, `tools`, `tool_calls`, `tool_choice`, `stream`).
- If a reserved key is present in overrides, it MUST be ignored.

### Reasoning outputs
Providers vary:
- Some return reasoning in a separate field (e.g., `reasoning_content`, `thoughts`).
- Some embed “thinking” in the main content.

Sisyphus will map provider-specific reasoning output into:
- `reasoning_summary`: safe to display and optionally store
- `reasoning_raw`: debug-only, never stored by default

Summary provenance:
- If a provider returns a dedicated summary field, map it directly.
- If a provider only returns raw reasoning, the system does not synthesize a summary in this change.

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
To avoid breaking existing event consumers, reasoning-related output SHOULD be sent as `SystemEvent::MessageReceived` with `role = "system"` and a structured discriminator.

Proposed event shape:
- Extend `SystemEvent::MessageReceived` payload with an optional `kind` field.
- For reasoning summary emission, set `kind = "reasoning_summary"`.
- For debug-only raw reasoning emission, set `kind = "reasoning_raw"`.

Rationale:
- The server currently broadcasts `SystemEvent` objects as JSON over SSE without versioning.
- Adding new enum variants is feasible, but increases client coupling.
- Using a discriminator avoids sentinel content prefixes and enables the TUI to toggle visibility without parsing content.

## Slash command integration
The system SHALL support a `/think` command to toggle reasoning summary output for the current session.

Behavior:
- `/think` toggles a session-scoped preference that controls whether the agent emits `kind = "reasoning_summary"` events.
- `/think` does not enable raw reasoning output.
- The TUI additionally uses `/think` to toggle local rendering of `kind = "reasoning_summary"` transcript entries.

## Security notes
- Never treat reasoning text as instruction authority for tool execution.
- Never include raw reasoning in tool arguments.
- Redact sensitive material in any debug-visible reasoning payloads.
