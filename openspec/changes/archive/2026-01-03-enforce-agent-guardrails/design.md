# Design: Enforce agent guardrails in core

## Goals
- Make permissions effective at runtime for tools and other privileged operations.
- Ensure the system prompt matches the project spec and is deterministic within a single user turn.
- Keep slash command UX consistent and discoverable.
- Ensure session lifecycle behaviors match spec (busy/idle, clear/new semantics).

## Non-Goals
- Introducing new capabilities beyond guardrail enforcement and correctness fixes.
- Changing any code under `opencode/`.

## Proposed Runtime Model

### Permission Enforcement
- Treat permissions as a runtime gate at the boundary where actions occur.
- For tool execution:
  - Allow: execute tool normally.
  - Ask: do not execute automatically; emit a permission-request signal/event and return a deterministic response.
  - Deny: do not execute; return a deterministic permission-denied response.

#### Deterministic User-Facing Responses
- Deny response (exact message content):
  - `Permission denied: tool execution is set to Deny.`
- Ask response (exact message content):
  - `Permission required: approve tool execution to continue.`

#### Deterministic Permission Request Signal
- When permission is Ask, the runtime emits a single permission-request signal with the following fields:
  - `operation`: `tool_execution`
  - `tool_name`: the requested tool name
  - `call_id`: the tool call id

This proposal does not define resume/replay semantics beyond emitting this signal and returning the deterministic message.

### System Prompt Construction
- Build the system prompt once per user input.
- Include:
  - Agent instructions
  - `<env>` block containing OS, CWD, and Date
  - Workspace custom rules appended from `AGENTS.md` when present

#### Determinism Boundaries (Snapshot Rules)
- Snapshotted once at user-turn start:
  - Working Directory
  - Platform (OS)
  - Today's Date
  - `AGENTS.md` existence and content
- The snapshotted values MUST be used for every completion request in that user turn, even if subsequent tool calls change process state (e.g., working directory).

#### Prompt Assembly Order
1. Agent instructions
2. `<env>` block
3. `AGENTS.md` content (verbatim), if present

### Slash Commands
- Keep the existing separation where the loader performs file I/O and the registry stores command metadata.
- Generate `/help` output from the registry to prevent drift.
- Parse arguments with support for quoted strings so commands can accept multi-word parameters.

#### Help Output Ordering
- `/help` lists commands in ascending lexicographic order by command name.

#### Argument Parsing Grammar (Minimal)
- Split on ASCII whitespace.
- Double quotes (`"`) group multi-word values into a single argument.
- Inside a quoted argument, `\"` represents a literal `"`, and `\\` represents a literal `\`.
- An unterminated quote is a parse error.

### Command Effects and Session State
- Keep command handlers pure (return outcomes/effects) and have the agent runtime apply effects to the session context consistently.
- Ensure busy/idle transitions remain correct even in early-return paths.

#### Session Context Definition (For Clear/New)
- “Session context” means the conversation history and tool-result history used to build the next completion request.
- Clearing context does not unload the slash command registry.

## Compatibility and Migration
- Preserve existing command names and behaviors, except where behavior is currently non-compliant with specs (permissions not enforced, `/help` drift, missing prompt rule injection).
