# Interactive CLI Command Menu Design

## Architecture

The implementation spans the Server, Client, and CLI components to propagate command information to the user interface.

### 1. Server-Side Changes (`crates/server`)

We need to expose the registered commands to the client.

- **Endpoint**: `GET /api/v1/commands`
- **Response**: JSON array of command objects.
  ```json
  [
    { "name": "help", "description": "Show help message", "type": "builtin" },
    { "name": "exit", "description": "Exit the session", "type": "builtin" },
    { "name": "my-cmd", "description": "A custom agent command", "type": "custom" }
  ]
  ```
- **Implementation**:
  - Add `list_commands` method to `Agent`.
  - Add route handler in `server/src/lib.rs`.

### 2. Client-Side Changes (`crates/client`)

- **Struct**: Define `CommandInfo` struct matching the JSON response.
- **Method**: Add `get_commands(&self) -> Result<Vec<CommandInfo>>` to `Client`.

### 3. CLI Changes (`crates/cli`)

We will replace the `std::io::BufReader` loop with `reedline`, a robust line editor library (used by Nushell).

#### Dependencies
Add to `crates/cli/Cargo.toml`:
```toml
[dependencies]
reedline = "0.28" # Check for latest version
```

#### Components

1.  **`CommandCompleter`**:
    - Implements `reedline::Completer`.
    - Fetches commands from the server (cached).
    - Returns `Suggestion`s when the input starts with `/`.

2.  **`Reedline` Configuration**:
    - Initialize `Reedline` with `CommandCompleter`.
    - Configure an `IdeMenu` (vertical list menu).
    - **Keybinding**:
        - To satisfy "hit '/' to bring up menu", we can configure the `Completer` to be aggressive or bind the `/` key.
        - *Recommendation*: Use `reedline`'s completion system. When the user types `/`, the completer offers candidates. We can configure the `IdeMenu` to be triggered by a specific key (like Tab) or try to auto-trigger.
        - *Specific Requirement "Hit / ... bring up menu"*: We can define a `Keybinding` for `/` that performs `InsertChar('/')` followed by `Menu("command_menu")`. This ensures the menu pops up immediately.

#### Interaction Flow

1.  User starts CLI.
2.  CLI fetches commands from server in background.
3.  User presses `/`.
4.  `Reedline` inserts `/`.
5.  The keybinding triggers `ReedlineEvent::Menu("command_menu")`.
6.  The `IdeMenu` appears with the list of commands (filtered by "empty string" initially, or just all commands).
7.  User uses Up/Down arrows to highlight.
8.  User presses Enter to select. The command name is appended to the `/`.
9.  User types arguments or presses Enter again to execute.
