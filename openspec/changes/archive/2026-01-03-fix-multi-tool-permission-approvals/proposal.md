# Change: Fix multi-tool permission approvals

## Why
The agent currently stops on the first Ask-gated tool call and drops any subsequent tool calls from the same assistant message. This violates tool-calling contracts (all tool calls in a message must be resolved) and can cause provider errors or hallucinated follow-ups due to missing tool outputs.

The current TUI permission overlay also assumes a single in-flight approval. If multiple permission requests arrive back-to-back, earlier requests become inaccessible.

## What Changes
- Ensure the agent processes every tool call in a single assistant message: execute Allow tools, produce deterministic results for Deny tools, and enqueue Ask tools as pending approvals.
- Ensure the agent does not perform the next model call until all tool calls from the current assistant message have corresponding Tool results (including deny results and approved tool outputs).
- Ensure the TUI can safely handle multiple PermissionRequest events by queueing them and presenting them deterministically.

## Impact
- Affected specs: agent-core, cli-tui
- Affected code: crates/core/src/agent.rs; crates/cli/src/ui/tui/{state.rs,mod.rs}
- Related work: openspec/changes/add-parallel-tool-runtime (multi-tool execution ordering)

