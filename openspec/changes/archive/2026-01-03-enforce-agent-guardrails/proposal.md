# Change: Enforce agent guardrails in core

## Why
The current core agent defines permissions and dynamic prompt behaviors, but key guardrails are not enforced consistently. This creates security and correctness gaps (e.g., permission settings have no effect, prompt rules are not injected, and command effects are not applied to session state).

## What Changes
- Enforce agent permission levels for tool execution.
- Generate the system prompt once per user turn and keep it stable across tool-call loops.
- Inject environment context in a structured `<env>` block and append workspace rules from `AGENTS.md` when present.
- Make `/help` output reflect the command registry with deterministic ordering, and improve slash command argument parsing.
- Apply command effects (clear history, new session) consistently to session state.

## Scope Notes
- This proposal focuses on the highest-ROI guardrails that can be made externally observable and testable without broad architectural changes.
- Approval/resume workflows for permission prompts are intentionally limited to a deterministic “approval required” signal and user-visible message; full resume semantics can be layered later.

## Impact
- Affected specs: agent-core, session-core, slash-commands
- Affected code: crates/core (agent runtime, prompt builder, command builtins, session lifecycle)

