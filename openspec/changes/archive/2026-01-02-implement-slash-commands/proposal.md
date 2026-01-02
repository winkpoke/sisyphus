# Implement Slash Commands

## Summary
Implement a Slash Command system in the Sisyphus core agent to support both built-in commands (e.g., `/help`) and custom template-based commands defined in `.sisyphus/command/*.md` files. This aligns Sisyphus with the `opencode` command architecture.

## Motivation
Slash commands are a primary interaction mode for the agent, allowing users to trigger specific workflows, load context, or execute tools quickly. Porting this functionality is essential for feature parity with `opencode`.

## Built-in Commands
The following built-in commands shall be implemented:
1. `/exit` and `/quit`: End the sessions and exit the program.
2. `/new`: Create a new session.

## Scope
- **Core**: `crates/core` (Agent, Command Registry, Command Logic).
- **Common**: `crates/common` (Command Types/Configuration).
- **Tooling**: Parsing of markdown command files.
