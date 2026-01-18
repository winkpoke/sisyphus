# Change: Enhance TUI UX to Match Modern Standards (Codex-style)

## Why
Current TUI uses "floating box" layout with heavy borders that reduces content visibility and feels dated. Modern AI coding assistants (Codex CLI, Claude Code) use immersive, pane-based designs.

## What Changes
- Switch to full-screen layout (remove outer margins)
- Introduce persistent ContextBar header (brand, working directory, active model)
- Replace plain text prefixes with styled MessageBlocks (headers + content + separators)
- Redesign Input/Status into unified footer with minimalist prompt (`> `)
- Add accessibility support (colorblind-friendly border patterns)
- Define minimum terminal size (24×80) with degraded mode fallback
- Optimize streaming performance (partial re-renders, 60 FPS batching)
- Add CWD tracking mechanism

## Impact
- **Affected specs**: `cli-tui/`
- **Affected code**: `crates/tui/src/tui/app.rs`, `state.rs`, `theme.rs`, `ui/mod.rs`, `ui/components/transcript.rs`, `ui/components/input.rs`, `ui/components/status.rs`
- **Breaking changes**: None (visual-only enhancement)
- **Accessibility**: Color differentiation augmented with border styles and patterns for colorblind users
- **Performance**: Minimal impact with efficient re-rendering during streaming
- **Migration**: Seamless (no state format changes)
