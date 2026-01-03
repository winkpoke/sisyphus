## Context
The agent runtime supports tool calling, including Ask-gated tools that require explicit user approval. Today, tool calls are processed sequentially and the turn is aborted immediately on the first Ask tool call. This drops any later tool calls from the same assistant message.

The TUI displays PermissionRequest events as a single overlay bound to one call_id. If more than one request arrives, the overlay state is overwritten.

## Goals / Non-Goals
- Goals:
  - Resolve every tool call in an assistant message deterministically.
  - Support multiple Ask-gated tool calls in a single assistant message without provider contract violations.
  - Ensure the TUI can present and action multiple pending approvals reliably.
- Non-Goals:
  - Introducing a batched approval UI (single decision for many calls).
  - Changing server APIs or adding new endpoints.
  - Parallelizing tool execution beyond existing/active work.

## Decisions
- Decision: Treat an assistant message with tool calls as an ordered sequence.
  - The agent MUST process tool calls in the strict order defined by the LLM.
  - Rationale: Tool calls often have causal dependencies (e.g., create file then write to it). Out-of-order execution violates this dependency.

- Decision: When executing a tool batch, the agent stops at the first `Ask` tool call.
  - `Allow` calls preceding the first `Ask` call are executed immediately.
  - The first `Ask` call emits a `PermissionRequest` and blocks the batch.
  - Subsequent tool calls (whether `Allow` or `Ask`) are HELD pending the resolution of the blocking call.
  - The agent returns a "permission required" response immediately upon blocking.

- Decision: Approval resolution resumes the batch execution.
  - When the user approves a call_id, the agent executes that tool.
  - The agent then continues processing the remaining tools in the batch.
  - If another `Ask` call is encountered, the process repeats (emit request, block).
  - The model is only called again once *all* tools in the batch are resolved/executed.

- Decision: The TUI maintains a FIFO queue of pending PermissionRequest events.
  - New PermissionRequest events enqueue.
  - The overlay always presents the oldest pending request.
  - After Approve/Deny, the queue advances and the next pending request is presented.

## Risks / Trade-offs
- Risk: UI/UX complexity increases slightly to handle a queue rather than a single request.
  - Mitigation: Keep the UI behavior deterministic and minimal (FIFO; no reordering).

- Risk: Some providers/models may produce mixed tool calls where later calls depend on earlier Ask-gated calls.
  - Mitigation: Preserve original tool-call order; only execute Allow/Deny immediately; Ask calls remain blocked and are resolved in order.

## Migration Plan
1. Update agent tool-batch execution semantics and add regression tests.
2. Update TUI state to queue PermissionRequest events and add state-level tests.
3. Validate end-to-end by exercising multi-tool messages that include multiple Ask calls.

## Open Questions
- None.

