# Change: Add Codex-style TUI foundation for Sisyphus CLI

## Why
The current CLI UI is a line-editor REPL that prints directly to stdout/stderr, which limits UX polish (streaming, structured transcript, overlays) and risks output interleaving as event streaming grows.

Codex demonstrates a robust pattern for an interactive agent CLI: a single async event loop that merges terminal input with backend events and renders a consistent UI frame.

## What Changes
- Add a new terminal UI frontend (“TUI”) for interactive chat sessions.
- Introduce a feature-gated/flagged UI selection so the existing REPL path remains available.
- Establish strict UI-layer boundaries so terminal rendering is not corrupted by incidental prints.

## Impact
- Affected specs: cli-architecture (modified), cli-tui (added)
- Affected code (expected): crates/cli (ui + commands), client event streaming plumbing
- Relationship to existing work: this change supersedes the REPL-only direction in change `implement-cli-interactive-menu` by establishing a full TUI frontend; that change should be paused or revised to target the new TUI surface.

## Non-Goals
- Implement full Codex-level transcript rendering (markdown, selection, copy, overlays).
- Implement command palette / completion popups beyond minimal scaffolding.

