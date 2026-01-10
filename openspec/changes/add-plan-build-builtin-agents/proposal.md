# Change: Add Plan and Build built-in agents

## Why
Sisyphus currently boots with a single built-in Agent instance, which forces planning and execution behaviors into one configuration. This makes it harder to enforce safety (read-only planning) while preserving an efficient implementation workflow.

Providing two built-in agents with distinct purposes enables:
- A read-only planning agent for architecture, sequencing, and risk analysis.
- A build agent for code changes and command execution with permission guardrails.

## What Changes
- Add two built-in agents addressable by stable ids: `plan` and `build`.
- Add `id` field to `AgentConfig` for stable routing identifiers (separate from display `name`).
- Make `plan` the default agent when no agent id is specified.
- Enforce that the Plan agent is read-only by (a) tool exposure limits and (b) permission policy.
- Ensure Agent discovery API lists both agents in a default server runtime.
- Keep both agents derived from the same runtime LLM configuration (provider/model/base_url).
- Add tool validation helper for Plan agent to prevent accidental registration of state-changing tools.

## Current Behavior (Gap)
- The core architecture already supports multiple agents (registry, session routing, server discovery).
- The default runtime only initializes one agent instance, so discovery and selection do not provide useful choices.
- `AgentConfig` lacks an `id` field, using `name` for both display and routing, which creates instability.
- `AgentRegistry::new()` derives registry key from `config.name` instead of a stable `id`.

## Impact
- Affected specs: agent-core, server-core
- Affected code (expected, apply stage):
  - `crates/core/src/agent/config.rs` - Add `id` field to `AgentConfig`
  - `crates/core/src/agent/registry.rs` - Update `new()` to use `config.id`
  - `crates/cli/src/bootstrap.rs` - Create two agents with distinct tool sets and configs
  - `crates/cli/src/commands/serve.rs` - Update to initialize multiple agents

## Non-Goals
- Adding new tools or changing existing tool schemas.
- Designing sub-agent orchestration between Plan and Build.
- Adding a CLI UX for agent selection beyond existing server/session agent_id selection.
- Changing to overall multi-agent architecture (registry, routing, discovery are already working).

