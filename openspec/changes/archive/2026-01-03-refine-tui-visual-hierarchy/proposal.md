# Refine TUI Visual Hierarchy & Layout

## Summary
Enhance the CLI TUI by introducing a structured status bar, dynamic headers, and improved whitespace to create a more grounded and informative workspace.

## Why
The current TUI layout is functional but bare-bones. It lacks visibility into system state (connection, processing status) and context (active model, session ID). The visual presentation is also cramped. A better layout will improve usability and user confidence.

## What Changes
- Replace the simple footer with a comprehensive Status Bar.
- Add "breathing room" to the Transcript view.
- Make the header dynamic to reflect the current context.

## Non-Goals
- Changing the underlying event loop logic (except for UI state updates).
- Adding new user input modes.
