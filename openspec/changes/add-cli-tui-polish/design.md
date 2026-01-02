## Context
The TUI will become the primary user-facing surface for interactive sessions, so it must provide discoverability and ergonomic controls without breaking the core guarantee: input remains safe while streaming.

## Goals / Non-Goals
- Goals:
  - Command palette with keyboard-first navigation.
  - Overlays for help/errors that do not corrupt terminal state.
  - Transcript selection and copy.
  - Clear key-hints and status indicators.
- Non-Goals:
  - Full onboarding wizard.

## Decisions
- Decision: Use a modal overlay system inside the TUI render loop rather than printing help to stdout.
- Decision: Palette triggers on leading `/` and can be explicitly opened via a keybinding.
- Decision: Copy uses OS clipboard when available; otherwise falls back to emitting a selectable text block in a pager.

## Risks / Trade-offs
- Clipboard support varies across OS/terminals.
  - Mitigation: layered fallbacks and clear error messaging.

## Migration Plan
1. Add palette and overlays first (low risk, local state).
2. Add selection/copy next, then optional enhancements (mouse selection, multi-click).

