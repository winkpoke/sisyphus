# Slash Command Architecture

## Overview
The Slash Command system serves as an interception layer before the main LLM chat loop. It parses user input for the `/` prefix and routes execution to either a compiled Rust function (built-in) or a template expansion logic (custom).

## Components

### 1. Command Registry (`CommandRegistry`)
A central repository stored within the `Agent` struct.
- **Responsibility**: Map command names (strings) to `CommandType`.
- **Storage**: `HashMap<String, CommandType>`.

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
   - **Built-in**: Execute the function immediately. Return the result (or feed result back into chat history?). *Decision: Built-ins typically perform actions or return info to the user immediately.*
   - **Custom**: Load the `template` from the config. Replace placeholders (if any) with arguments. Recursively call `Agent::chat` with the expanded prompt.

## Data Persistence
- Custom commands are loaded from `.opencode/command/**/*.md` at Agent startup.
- We will use a markdown parser (e.g., `gray_matter` equivalent) to separate Frontmatter (YAML) from Content (Template).

## Dependencies
- Need a crate for parsing YAML frontmatter from Markdown files.
