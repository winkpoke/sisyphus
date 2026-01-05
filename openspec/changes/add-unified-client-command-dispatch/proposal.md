# Change: Unified Client Slash Command Dispatch (Local + Remote)

## Why
The CLI currently has inconsistent slash-command behavior across frontends:
- The REPL forwards slash commands to the server for parsing/execution.
- The TUI partially executes commands locally using a separate registry and whitespace splitting.

This split causes drift in parsing semantics (quoted args), command availability/discovery, and lifecycle/effect handling.

## What Changes
- Introduce a unified client-side slash-command dispatch interface used by both REPL and TUI.
- Allow clients (REPL/TUI) to register local slash commands that run first and may optionally forward a rewritten message to the server.
- Make command discovery in the TUI command palette a merged view of remote commands (server-registered) and client-local commands.
- Define deterministic precedence and collision behavior between local and remote command names.

## Non-Goals
- Changing core slash-command parsing rules or custom command expansion behavior.
- Changing server-side command registry semantics or command effect meanings.
- Implementing new UI features beyond unifying dispatch and discovery behavior.

## Impact
- Affected specs:
  - cli-architecture
  - cli-tui
  - slash-commands
- Affected code (implementation stage):
  - CLI input routing for REPL and TUI
  - TUI command palette population
  - Client-side parsing to align with the core parser

