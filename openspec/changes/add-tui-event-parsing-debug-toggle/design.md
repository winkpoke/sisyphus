## Context
The server exposes an SSE stream at `/api/v1/events` and the TUI subscribes to it. The SSE payload data is JSON for a tagged `SystemEvent` enum.

Today the TUI handles `PermissionRequest` as an overlay, but still appends the raw JSON payload to the transcript. For other event types it also appends the raw JSON payload. This is not end-user-friendly.

Additionally, some events may carry sensitive strings (tool output, provider errors, or user content). A debug feature that prints raw payloads must be safe by default.

## Goals / Non-Goals
- Goals:
  - Render backend events in a concise, readable format for end users.
  - Preserve access to the raw event payloads via a local `/debug` toggle.
  - Ensure raw payload display is safe by default through redaction and truncation.
  - Never crash the TUI if an event payload is invalid or unknown.
- Non-Goals:
  - Changing the server event format or adding new server endpoints.
  - Defining a stable public JSON schema for third-party consumers.
  - Implementing a full event inspector UI beyond transcript lines and optional overlays.

## Decisions
- Decision: Treat the server event stream as a structured `SystemEvent` channel.
  - The TUI SHOULD attempt to parse every SSE message `data` into `SystemEvent`.
  - If parsing fails, the TUI MUST fall back to a safe, minimal representation.

- Decision: Default display is parsed summaries, not raw JSON.
  - For recognized event types, the transcript SHOULD show a one-line summary suitable for end users.
  - For `PermissionRequest`, the overlay remains the primary UI; the transcript SHOULD NOT also show the raw JSON payload.

- Decision: `/debug` is a local TUI command.
  - The toggle MUST not be sent to the server as a chat message.
  - The toggle MUST persist for the duration of the TUI session.

- Decision: Raw event payload display is always redacted and truncated.
  - Debug mode MUST redact sensitive fields in JSON (e.g., keys containing `token`, `secret`, `api_key`, `authorization`) and common bearer-token patterns.
  - Debug mode MUST truncate output to a fixed maximum size to prevent transcript flooding.

## Rendering Policy (initial mapping)
- `ToolExecuted`: show a system line indicating the tool completed; optionally include a short, truncated result preview.
- `Error`: show an error line.
- `AgentStateChanged`: show a short system status line.
- `MessageReceived`: do not duplicate user/assistant chat content that is already shown via the primary chat UI path.
- Unknown or unparseable: show a short system line; show raw payload only when debug is enabled.

## Risks / Trade-offs
- Risk: Redaction may hide information needed for debugging.
  - Mitigation: Only redact sensitive values; keep key names and structural context.

- Risk: Event suppression (e.g., `MessageReceived`) could hide useful signals.
  - Mitigation: Allow raw payload viewing in debug mode and keep summaries for high-signal events.

## Migration Plan
1. Introduce a small event-to-display mapping in the TUI.
2. Add a TUI-local debug flag and `/debug` command.
3. Add unit tests for parsing, redaction, and truncation behavior.
4. Validate behavior with server streaming and permission prompt flows.
