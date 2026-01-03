# Change: Add permission prompt UX to the CLI TUI and hard-stop on Ask

## Why
When agent permissions are set to `Ask`, tool execution is blocked, but the agent currently continues the turn after receiving the deterministic “permission required” tool result. This causes the model to guess and produce incorrect answers instead of stopping for user action. Additionally, the CLI TUI currently renders backend events as raw text, so users have no clear, discoverable indication that an approval is required.

## What Changes
- Hard-stop the current chat turn when a tool call requires approval (`Ask`), returning the deterministic “permission required” message without further model calls.
- Decode `SystemEvent` payloads from the server event stream and render `PermissionRequest` as a first-class UI prompt.
- Add a permission-required overlay in the CLI TUI that clearly indicates the agent is blocked.

## Impact
- Affected specs: cli-tui, server-core, agent-core
- Affected code: crates/cli (TUI event handling and UI state), crates/server (event stream payloads), crates/core (agent turn loop stop semantics)
