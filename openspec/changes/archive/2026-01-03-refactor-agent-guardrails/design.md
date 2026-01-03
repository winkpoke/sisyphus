# Design: Refactor Agent Guardrails

## Goals / Non-Goals
- Goals:
  - Eliminate blocking I/O in the agent runtime when building the system prompt.
  - Make rule-file discovery deterministic by anchoring it to the workspace root.
  - Decouple slash command parsing from agent execution flow and improve testability.
  - Make tool permission policy configurable without losing safe defaults.
- Non-Goals:
  - Introduce new tools or change tool schemas.
  - Change command semantics beyond making parsing and expansion more reliable.

## Problem
1. **Blocking I/O**: `SystemPromptBuilder::snapshot()` uses `std::fs` to read `AGENTS.md`. In the async `tokio` runtime, this blocks the worker thread, potentially causing latency spikes or deadlocks in high-throughput scenarios.
2. **Coupled Parsing**: The `Agent` struct contains ad-hoc string parsing logic (`parse_command_args`) for slash commands. This violates the Single Responsibility Principle and makes the agent harder to test and maintain.
3. **Hardcoded Permissions**: `Agent::get_permission_level` contains a `match` statement hardcoding tools like "run_command" to specific permission levels. Adding new tools or changing policies requires modifying the code structure rather than configuration.

4. **Implicit Workdir Semantics**: Custom rules are discovered relative to the process working directory, which may differ from the intended workspace root used by tools.

5. **Config Is Not Runtime-Wired**: A “configurable permissions” model is incomplete unless the running CLI/server actually loads and applies that configuration.

## Solution

### Async Prompt Builder
- Change `SystemPromptBuilder::snapshot()` to an async function.
- Use `tokio::fs::read_to_string` instead of `std::fs`.
- Snapshot once per user turn and reuse it across any internal completion/tool-call loop within that turn.

### Workspace Root Resolution
- Define “workspace root” as the same root enforced by the tool sandbox.
- Resolve `AGENTS.md` from that workspace root, not from an implicit process CWD.
- If `AGENTS.md` is missing, omit custom rules rather than failing the turn.

### Command Parser Extraction
- Create a new module `crates/core/src/command/parser.rs`.
- Move `parse_command_args` and its tests there.
- Expose a stable API that supports both execution and custom command expansion:
  - `cmd`: the command name token (e.g., `/clear`)
  - `args`: parsed, dequoted argument vector
  - `raw_args`: the raw substring after the command token (trimmed), preserving original quoting/escapes

### Dynamic Permission Configuration
- Preserve category-level defaults (edit/bash/skill) and add per-tool overrides by tool name.
- Permission evaluation order:
  1. If `permissions.overrides[tool_name]` exists, use it.
  2. Else map tool name to a category (bash/edit/skill) and use that category’s default.
  3. If a tool is not recognized by the category map, treat it as `skill`.
- Safety invariant: default category permissions MUST be safe by default (Ask/Deny) so new tools cannot silently become allowed.

### Configuration Wiring
- Load agent permissions (and optionally command_path) from the same configuration entrypoint used by the running binary.
- Ensure backward compatibility by defaulting to safe values when configuration is absent.

## Migration Strategy
- Apply changes incrementally.
- Ensure existing tests pass (especially the new guardrail tests).

## Risks / Trade-offs
- Making workspace root explicit may surface environments where CWD previously differed; mitigate by defining and testing root resolution.
- Per-tool overrides must not weaken defaults; mitigate by requiring safe defaults and explicit overrides.
