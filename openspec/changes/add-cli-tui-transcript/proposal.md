# Change: Add Codex-like transcript and streaming to the Sisyphus TUI

## Why
“Codex-like polish” requires that the CLI shows a structured, progressively-updating transcript (assistant streaming, tool/agent activity) without disrupting input. A minimal TUI foundation is not sufficient without a robust transcript model, scrolling behavior, and predictable rendering.

## What Changes
- Add a transcript model suitable for progressive updates (streaming deltas and discrete events).
- Implement scrolling and “stick to bottom” behavior while preserving input focus.
- Define a stable rendering contract for transcript rows (wrapping, truncation, width changes).

## Impact
- Affected specs: cli-tui (modified), cli-architecture (modified)
- Affected code (expected): crates/cli UI state/rendering; event typing for SSE → UI

## Non-Goals
- Rich selection/copy UX and overlays (handled in a later polish change).
- Advanced markdown rendering beyond what is required for a readable transcript.

