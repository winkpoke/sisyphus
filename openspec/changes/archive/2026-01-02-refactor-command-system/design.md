# Command System Redesign

## Context
The current command system is implemented as:
- A `CommandRegistry` mapping command names to either a synchronous built-in closure or a custom markdown template.
- An `Agent::chat` pre-loop that repeatedly expands custom commands by rewriting the input, and executes built-ins directly.

This creates concrete problems:
- `/new` currently clears history in-place and cannot create a new Session ID.
- `/exit` is implemented via an event bus, while the CLI REPL also intercepts `/exit` and exits without calling the server.
- Built-ins are synchronous closures, which makes IO-bound commands awkward.

## Goals / Non-Goals
- Goals:
  - Support stateful and asynchronous built-in commands.
  - Provide structured lifecycle signaling so the caller applies session lifecycle changes.
  - Preserve custom markdown command loading and template expansion.
  - Make `/new` and `/clear` semantics explicit and correct.
- Non-Goals:
  - Redesign the LLM turn loop and tool execution.
  - Introduce a new permission model for slash commands.

## Architecture

### Command Interface
We move from `Fn` closures to a trait to allow stateful commands and async execution.

The command interface MUST avoid holding `&mut Session` across await points. To enforce that, commands return an intent (effect) that the host applies.

```rust
#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, ctx: CommandContext, args: CommandArgs) -> Result<CommandOutcome>;
}
```

`CommandContext` is a lightweight view of runtime services and session identity (e.g., session id, config, event bus handle), not a mutable borrow of the session state.

`CommandArgs` is the centrally parsed argument list; commands MUST NOT re-parse the raw input.

### Command Outcome
Commands communicate both user-visible output and lifecycle intent via a structured return.

```rust
pub struct CommandOutcome {
    pub output: Option<String>,
    pub effect: CommandEffect,
}

pub enum CommandEffect {
    None,
    ClearHistory,
    NewSession,
    Exit,
}
```

### Lifecycle Flow
The `Agent::chat` entry point returns a chat response that includes the lifecycle effect so the caller can apply it.
- `None`: Normal behavior.
- `ClearHistory`: The caller clears history for the current session ID.
- `NewSession`: The caller creates a new session and returns its ID to the client.
- `Exit`: The caller initiates shutdown.

### Dependency Injection
`CommandContext` provides access to services (e.g., event publishing) but lifecycle control flow MUST be expressed via `CommandOutcome.effect`.

## Custom Markdown Commands
Custom markdown commands remain a template-expansion stage:
- On startup, load markdown files from the configured command directory into the registry.
- On invocation, expand the template with `{{args}}` and re-run routing on the expanded input.
- Apply an explicit recursion limit to prevent cyclic expansions.

Template expansion MUST be applied before LLM invocation.

## Server & Client Implications
To fully support `NewSession` where the client switches to a new Session ID:
1. **Server**: The chat endpoint MUST return a response that includes both the assistant output and an optional updated session id.
2. **Client/CLI**: The client MUST update its stored session id when the server indicates a session switch.

## Migration Strategy
1. Define the new command interface and outcome/effect types.
2. Refactor the built-in command registry to store command objects.
3. Preserve custom markdown command loading and expansion in the routing layer.
4. Update `Agent::chat` to return a structured response containing output plus lifecycle effect.
5. Update the server chat handler to apply lifecycle effects using `SessionManager`.
6. Update the client and REPL to adopt returned session IDs.

## Risks / Trade-offs
- Keeping custom template expansion enables flexible local workflows but requires recursion limits and careful lifecycle semantics.
- Returning lifecycle effects to the host reduces coupling but requires minor API surface changes on server/client boundaries.

## Open Questions
- Should template expansion be allowed to produce lifecycle commands (e.g., a custom command expanding into `/exit`)?
- Should argument parsing support quotes and escapes, or remain whitespace-split for now?
