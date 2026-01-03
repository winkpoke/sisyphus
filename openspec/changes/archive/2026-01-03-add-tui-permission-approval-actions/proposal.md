# Change: Add approve/deny actions for Ask-gated tool execution in the CLI TUI

## Why
When tool execution permissions are set to `Ask`, the system blocks tool execution and emits a `PermissionRequest`, but users cannot currently approve or deny within the CLI TUI. This makes the flow confusing and forces users to change configuration and retry, instead of making an in-context decision.

## What Changes
- Add an explicit approve/deny workflow for `PermissionRequest` events.
- Add a server API to submit approval decisions correlated to a `call_id`.
- Extend the agent runtime to resume an in-progress turn after an approval decision:
  - On **Approve**, execute the pending tool call and continue the same assistant turn.
  - On **Deny**, write a deterministic denial tool-result (correlated to `call_id`) and continue the same assistant turn to produce a safe alternative.
- Update the CLI TUI permission overlay to provide discoverable controls for Approve and Deny.

## Impact
- Affected specs: agent-core, server-core, cli-tui
- Affected code: crates/core (agent turn control + resume path), crates/server (approval endpoint), crates/client (client method for approvals), crates/cli (TUI overlay controls + request wiring)

