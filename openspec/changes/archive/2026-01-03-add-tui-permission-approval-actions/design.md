# Design: Interactive tool permission approvals in the CLI TUI

## Goals
- Allow users to approve or deny Ask-gated tool execution directly in the CLI TUI.
- Make approval a first-class, deterministic part of a single assistant turn.
- Keep the implementation minimal and consistent with existing tool-call semantics (every tool call gets a tool result).

## Non-goals
- Session-wide approval caching (e.g., “approve for session”).
- A general authorization framework beyond the existing Allow/Ask/Deny gate.
- Remote multi-user permission delegation.

## Proposed Semantics

### Approve
- Executes the pending tool call identified by `call_id`.
- Appends the real tool-result message for that `call_id`.
- Continues the same assistant turn by performing the next model call.

### Deny
- Does not execute the tool call.
- Appends a deterministic tool-result message for that `call_id` with exact content:
  - `Permission denied: user rejected tool execution.`
- Continues the same assistant turn by performing the next model call, allowing the assistant to propose safe alternatives.

## Architecture

### Data model
- The system treats a `PermissionRequest` as a pending approval keyed by `(session_id, call_id)`.
- The server owns pending-approval tracking for each session.
- The agent remains stateless with respect to approvals by reconstructing state from the session history and the approval decision.

### Control flow
1. The model emits an Assistant message containing tool calls.
2. The runtime evaluates permissions and encounters `Ask` for a tool call.
3. The runtime emits `SystemEvent::PermissionRequest { operation, tool_name, call_id }` and stops the current turn.
4. The CLI TUI displays a permission overlay with Approve/Deny actions.
5. The user selects Approve or Deny.
6. The CLI sends the decision to the server’s approval API.
7. The server resumes the turn:
   - Approve: executes the tool, appends tool-result, performs the next model call.
   - Deny: appends deterministic denial tool-result, performs the next model call.
8. The server returns the assistant response; the UI appends it to the transcript.

## Server API

### Endpoint
`POST /api/v1/sessions/:id/approvals/:call_id`

### Request body
```json
{ "decision": "approve" }
```
or
```json
{ "decision": "deny" }
```

### Response
Returns the same shape as chat responses so the UI can append the assistant output without special casing:
```json
{ "response": "...", "session_id": "..." }
```

## Concurrency and safety
- The server MUST reject approval submissions when there is no pending approval for `(session_id, call_id)`.
- The server SHOULD treat repeated submissions for the same `(session_id, call_id)` as idempotent.
- The system MUST keep the approval flow local to the current process boundary (no external network exposure requirement).

