## Context
Sisyphus currently runs an interactive REPL with `reedline` and prints responses directly. Codex’s TUI approach uses `crossterm` + `ratatui` and a single async event loop to render consistently while consuming backend events.

## Goals / Non-Goals
- Goals:
  - Introduce a TUI frontend with a stable, flicker-free render loop.
  - Keep REPL as a fallback path during migration.
  - Enforce separation so UI rendering is not corrupted by incidental stdout/stderr writes.
- Non-Goals:
  - Full transcript polish (selection/copy, markdown, overlays) in this change.

## Decisions
- Decision: Use `crossterm` for terminal mode + input events and `ratatui` for rendering.
- Decision: Single-owner async event loop merges terminal events and backend AppEvents.
- Decision: Gate TUI behind a config/flag and auto-disable when stdin/stdout are not TTY.

## Alternatives Considered
- Keep improving reedline-based REPL.
  - Rejected: reedline is optimized for line editing, not multi-pane UIs, overlays, or structured streaming transcripts.

## Risks / Trade-offs
- Terminal corruption risk if other parts of the process write to stdout/stderr while the TUI is active.
  - Mitigation: establish “no stdout/stderr from UI libraries” policy; route logs to files.
- Windows terminal differences (raw mode, mouse capture, key reporting).
  - Mitigation: treat advanced terminal features as best-effort; always restore terminal modes.

## Migration Plan
1. Add TUI foundation and UI selection.
2. Keep REPL as default until TUI achieves functional parity.
3. Flip default to TUI once transcript/streaming and command UX are ready.

## Open Questions
- Should TUI be the default for `sisyphus chat` once stable, or only behind a flag?
- Should the TUI use alternate screen, inline viewport, or both (like Codex overlays)?

