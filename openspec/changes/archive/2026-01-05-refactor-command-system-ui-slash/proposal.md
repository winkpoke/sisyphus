# Change: Split command system into UiCommand and SlashCommand

## Why
The current system uses slash-prefixed commands for multiple responsibilities (prompt templating, lifecycle, and UI-only actions), which creates inconsistent behavior across clients and makes command ownership unclear.

## What Changes
- Define two explicit command kinds:
  - **SlashCommand**: a prompt template expanded server-side and injected into the agent prompt.
  - **UiCommand**: a command handled by the client UI runtime (CLI/TUI/others), optionally calling server endpoints.
- Keep custom SlashCommand loading from the configured command directory as Markdown templates.
- Reject custom SlashCommands that collide with reserved UiCommand names.
- Make command discovery explicit:
  - Server exposes discoverable SlashCommand metadata from server configuration.
  - Clients merge SlashCommand metadata with local UiCommand metadata for palettes and help.
- Move lifecycle and UI-only commands (e.g. `/new`, `/clear`, `/exit`, `/debug`, `/help`) to UiCommand semantics.

## Impact
- Affected specs: `slash-commands`, `server-core`, `session-core`, `cli-architecture`, `cli-tui`
- Affected systems: core command expansion in agent, server APIs for discovery and session operations, client-side routing and command palette
- **BREAKING**: Slash commands no longer produce session lifecycle effects via chat; lifecycle is driven by UiCommands calling server endpoints.
