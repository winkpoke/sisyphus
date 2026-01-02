# Refactor Command System

## Summary
Refactor slash-command execution to support asynchronous built-ins, structured lifecycle signaling, and correct session lifecycle semantics (`/new` creates a new session ID; `/clear` clears history). Preserve existing custom markdown command loading and expansion.

## Why
- **Incorrect semantics**: `/new` currently clears history in-place and does not create a new Session ID, contradicting the slash-commands spec and user expectations.
- **Coupled side-effects**: `/exit` relies on an out-of-band event bus and the CLI also intercepts `/exit` locally, leading to inconsistent lifecycle behavior across modes.
- **No async path**: Built-in command handlers are synchronous closures, blocking support for IO-bound commands.
- **Parsing duplication**: Command parsing is ad-hoc and repeated; adding more commands will increase edge cases.

## What Changes
- Replace closure-based built-in handlers with a stateful command interface that supports asynchronous execution.
- Standardize command parsing/dispatch in one place and pass parsed arguments to commands.
- Introduce a structured lifecycle signal returned by commands so the caller applies session lifecycle changes.
- Define explicit semantics:
  - `/new` creates a fresh session (new ID, empty history)
  - `/clear` clears history for the current session (same ID)
  - `/exit` signals shutdown consistently
- Preserve custom markdown commands as a template-expansion stage with the existing recursion guard.

## Impact
- Affected specs:
  - `slash-commands` (lifecycle semantics, async execution, clear vs new)
  - `server-core` (chat response must convey session switch)
  - `cli-architecture` (REPL must adopt new session IDs)
- Affected code (planned in apply stage):
  - Core command routing and agent chat loop
  - Server chat handler response model
  - Client chat method and REPL session tracking

## Non-Goals
- Redesign of the overall agent turn loop or tool execution model.
- Authorization/permissions model for commands beyond existing trust boundaries.
- Full shell-style parsing beyond basic quoted argument support.

## Risks
- Async trait design must avoid holding `&mut Session` across await points.
- Session switching must be represented in the server response without breaking existing clients.
- Custom command expansion can create surprising lifecycle effects if not constrained.
