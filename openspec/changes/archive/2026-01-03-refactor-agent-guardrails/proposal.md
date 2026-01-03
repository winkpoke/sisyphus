# Change: Refactor agent guardrails for performance and maintainability

## Why
The current guardrails implementation blocks on synchronous file I/O in an async runtime, mixes slash-command parsing with execution flow, and encodes tool permission policy in code in a way that is hard to evolve safely. This change refactors guardrails to be non-blocking, testable, and policy-driven while preserving safe defaults.

## What Changes
- **Async Snapshot + Prompt Build**: Convert prompt snapshot construction to async file I/O to avoid blocking the agent runtime, and define snapshot timing as once per user turn (reused across any internal tool-call loop for that turn).
- **Workspace-Root Rule Resolution**: Resolve `AGENTS.md` from the agent workspace root (consistent with the tool sandbox root) rather than implicitly depending on the process CWD.
- **Command Parser Extraction**: Move command parsing out of `Agent` into a dedicated parser component with a stable API that returns both structured arguments and the raw argument tail for custom command expansion.
- **Safe, Configurable Permissions**: Replace hardcoded tool-name matching with a configuration-backed policy that preserves category defaults (edit/bash/skill) and adds per-tool overrides, with safe behavior for unspecified/unknown tools.
- **Configuration Wiring**: Make agent permission policy loadable via the existing configuration path (e.g., `sisyphus.toml`) so “configurable permissions” is real at runtime, not only a struct-level refactor.

## Scope Notes
- This change focuses on guardrails correctness and maintainability; it aims to preserve existing user workflows while clarifying previously implicit behavior (snapshot timing, rule file resolution, permission fallbacks).
- Primary focus is `crates/core`, with minimal wiring changes in config/bootstrap paths to ensure permissions can be set without code edits.

## Impact
- Affected specs: `agent-core`, `tooling`, `slash-commands`
- Affected code: `crates/core/src/agent.rs`, `crates/core/src/agent/prompt.rs`, `crates/core/src/agent/config.rs`
