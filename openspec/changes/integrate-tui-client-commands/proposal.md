# Change: Integrate TUI Client Commands (Without EventBus)

## Why
The core `Command` trait is text-oriented and works well for server-side slash commands and tools, but the TUI also needs UI-scoped commands (e.g., `/exit`, `/new`, `/clear`, `/debug`) that must trigger client-side behavior and/or client API calls.

Using `SystemEvent` / `EventBus` as a control plane for these UI effects is the wrong abstraction boundary: `SystemEvent` is used for backend SSE events and represents a serialized, cross-process protocol, while UI state changes should be expressed as local MVU actions/instructions.

## What Changes
- Route UI-scoped slash commands to explicit TUI instructions (MVU-style) instead of publishing `SystemEvent`s.
- Keep backend SSE `SystemEvent` focused on server-originated events; do not add UI-only variants like “SessionCreated”.
- Make command parsing and routing deterministic and consistent with server semantics by reusing the shared core command parser for slash inputs.
- Keep command discovery unified in the palette by merging local UI commands with remote commands from `/api/v1/commands`.

## Impact
- Affected specs: cli-tui, slash-commands
- Related change: add-unified-client-command-dispatch (shared routing model)
- Affected code (implementation phase): crates/cli/src/ui/tui (input routing + palette), crates/core/src/command/parser.rs (reuse only, no behavior change)
