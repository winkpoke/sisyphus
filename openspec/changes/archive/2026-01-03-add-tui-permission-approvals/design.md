# Design: Permission prompts and hard-stop semantics for Ask-gated tool execution

## Goals
- Prevent the agent from producing non-deterministic or incorrect output after an Ask-gated tool call.
- Make `PermissionRequest` externally observable and clearly visible in the CLI TUI.
- Keep the initial implementation minimal and deterministic.

## Non-goals
- Approving and resuming the exact paused tool call within the same assistant turn.
- Session-scoped permission overrides applied via a server API.
- A general authorization system beyond tool-execution permission gating.

## Current Behavior
- The agent enforces tool permissions with `Allow | Ask | Deny`.
- When permissions are `Ask`, tool execution is not performed; instead the agent emits `SystemEvent::PermissionRequest { operation, tool_name, call_id }` and returns a deterministic user-visible message.
- The server streams system events as JSON strings via SSE.
- The CLI TUI subscribes to SSE, but currently renders the raw JSON string as a transcript line and does not interpret permission requests.

## Proposed Behavior

### 0) Hard stop on Ask
When a tool call evaluates to `Ask`, the system already emits `SystemEvent::PermissionRequest { operation, tool_name, call_id }` and produces a deterministic tool result string.

The agent runtime SHALL treat this as a blocking condition for the current turn:
- It MUST stop executing further tool calls for the current completion response.
- It MUST NOT call the model again for that user turn.
- It MUST return the deterministic “permission required” message as the assistant-visible outcome for that turn.

### 1) Event decoding and UI prompt
The CLI TUI SHALL parse SSE message data as a JSON-encoded `SystemEvent`.

When a `PermissionRequest` is received, the TUI SHALL present an overlay showing:
- Operation (expected: `tool_execution`)
- Tool name
- Call id (for correlation/debug)

The overlay SHALL indicate that the agent is blocked awaiting user action.

## UX Sketch (CLI TUI)
- Permission overlay opens automatically on `PermissionRequest`.
- `Esc` closes the overlay.
- The UI keeps normal transcript and input intact; the overlay is a temporary interruption.

## Trade-offs
- This design prevents incorrect “continuation” outputs, but does not provide a built-in approval mechanism.
- Users will need to change permission policy (configuration or future UI) and retry the request.
