# Slash Command Architecture

## Overview
The Slash Command system serves as an interception layer before the main LLM chat loop. It parses user input for the `/` prefix and routes execution to either a compiled Rust function (built-in) or a template expansion logic (custom).

## Components

### 1. Command Registry (`CommandRegistry`)
A central repository stored within the `Agent` struct.
- **Responsibility**: Map command names (strings) to `CommandType`.
- **Storage**: `HashMap<String, CommandType>`.
- **Validation**: Command names must be valid (no whitespace) to ensure they can be parsed correctly.

### 2. Command Types
```rust
pub enum CommandType {
    Builtin(Box<dyn Fn(&mut AgentContext) -> Result<String>>),
    Custom(CommandConfig),
}

pub struct CommandConfig {
    pub description: Option<String>,
    pub template: String,
    // Future: agent, model, etc.
}
```

### 3. Execution Flow
1. **Input**: `Agent::chat(input)` receives a string.
2. **Check**: If `input.starts_with('/')`:
   - Parse command name and arguments.
   - Lookup in `CommandRegistry`.
3. **Branch**:
   - **Built-in**: Execute the function immediately. Return the result.
     - **Lifecycle Commands**: Commands like `/exit` and `/quit` trigger a `SystemEvent::Shutdown` on the event bus. The CLI (or other clients) must subscribe to this event to perform a clean exit, ensuring consistency between the Agent's state and the application process.
   - **Custom**: Load the `template` from the config. Replace placeholders (if any) with arguments. Recursively call `Agent::chat` with the expanded prompt.

## Data Persistence & Loading
- **Source**: Custom commands are loaded from a directory specified in `AgentConfig` (default: `.sisyphus/command`).
- **Resilience**: The loading process (`load_from_dir`) must be resilient.
  - Errors reading individual files (e.g., permissions, bad formatting) should be logged as warnings.
  - The loader should **continue** to the next file rather than aborting the entire registry initialization.
- **Parsing**: Use `gray_matter` to separate Frontmatter (YAML) from Content (Template).
- **Sanitization**: Filenames containing whitespace (e.g., `my command.md`) should be skipped or sanitized, as they cannot be invoked via the standard `/cmd args` syntax.

## Dependencies
- `gray_matter` for parsing YAML frontmatter.
- `serde` for deserializing config.
