# Design: Selective parallel tool execution runtime

## Overview
Add a `ToolCallRuntime` that executes tool calls with selective parallelism based on a per-tool execution mode.

## Execution Model

### Concurrency Gate
Use a single runtime gate implemented with an `RwLock`:
- Parallel tools acquire a read lock, allowing concurrent execution.
- Sequential tools acquire a write lock, blocking all other tools until completion.

### Tool Classification
Each registered tool is assigned an execution mode:
- Parallel: read-only operations that do not mutate workspace state and do not require serialized access to shared external systems.
- Sequential: operations that mutate workspace state, spawn processes, or interact with stateful external systems.

The default MUST be Sequential unless a tool is explicitly marked Parallel.

## Determinism
Even when tools execute concurrently, transcript construction MUST be deterministic:
- Tool results are appended to the session in the same order as the originating `tool_calls` list.
- If a tool fails, its error output is recorded in its corresponding tool result position.

## Safety Notes
- Parallel classification is an allowlist, not a heuristic.
- “Read-only” does not imply safe if the tool depends on mutable external state; such tools remain Sequential.
- The runtime SHOULD avoid starvation of sequential tools under heavy parallel load.

