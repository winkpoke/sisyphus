# Interactive CLI Command Menu

## Summary
Enhance the Sisyphus CLI with an interactive command menu that appears when the user types `/`. This menu will allow users to navigate available commands using arrow keys and select them, improving discoverability and usability.

## Motivation
Currently, the CLI uses a simple line-buffered input. Users must know the available commands (like `/help`, `/exit`, or custom agent commands) and type them exactly. This lacks modern CLI conveniences found in other tools.
Implementing an interactive menu will:
- Aid memory by listing available commands.
- Speed up interaction by allowing selection.
- Provide a foundation for future autocompletion features.

## Goals
- Expose available commands via the Server API.
- Implement a client method to fetch commands.
- Integrate `reedline` into the CLI to replace standard `stdin`.
- Configure `reedline` to trigger a menu on `/`.
- Support Up/Down arrow navigation and Enter selection.

## Non-Goals
- Full TUI application (we stick to a REPL with enhancements).
- Complex argument completion (for this iteration, just command names).
