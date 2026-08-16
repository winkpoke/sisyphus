# Change: Add selective parallel tool execution runtime

## Why
Tool calls are currently executed sequentially, which unnecessarily increases latency for read-only operations (e.g., glob/grep, file reads). Adding selective parallel execution improves responsiveness while keeping state-changing operations safe and deterministic.

## What Changes
- Introduce a tool execution runtime that supports parallel execution for explicitly safe tools.
- Enforce exclusivity for sequential tools to prevent unsafe interleavings with workspace mutations and external side effects.
- Preserve deterministic transcript behavior regardless of tool completion order.

## Impact
- Affected specs: agent-core, tooling, session-core
- Affected code: crates/core tool execution loop and tool runtime (new module expected)

