# Change: Add TUI event parsing and /debug toggle

## Why
The TUI currently displays backend SSE events as raw JSON in the transcript, which is noisy and hard to read for end users.

At the same time, raw event payloads are still valuable for troubleshooting, so the UI needs an explicit debug mode that can surface raw events safely.

## What Changes
- Parse backend SSE event payloads into structured `SystemEvent` values and render a concise, end-user-readable view by default.
- Add a local TUI command `/debug` to toggle showing raw event payloads.
- When debug mode is enabled, raw event payloads MUST be redacted and truncated to reduce accidental secret exposure and transcript spam.

## Impact
- Affected specs: cli-tui
- Affected code: crates/cli/src/ui/tui/mod.rs; crates/cli/src/ui/tui/state.rs; crates/cli/src/ui/tui/transcript.rs
- Related specs: server-core (SSE event delivery), common-infra (safe logging patterns)
