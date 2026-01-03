<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->


For overall product requirements and system architecture, refer to the [PRD](PRD.md).
For testing standards and strategies, refer to [TEST_STRATEGY.md](TEST_STRATEGY.md).

Key runtime behavior (normative in OpenSpec):
- Ask-gated tool execution emits a `PermissionRequest` and blocks the current assistant turn.
- Tool calls in a single assistant message are processed in order; later calls are held until the blocking call resolves.
- Deny appends a deterministic Tool result: `Permission denied: user rejected tool execution.`
- Clients render permission prompts (operation/tool_name/call_id), queue multiple requests FIFO, and submit approve/deny decisions.
- LLM Providers must use the shared `SSEParser` (`crates/provider/src/sse.rs`) for streaming to ensure correct handling of split network chunks and multi-byte characters.

CLI TUI requirements (see `spec/cli-tui` in OpenSpec):
- Render backend SSE `SystemEvent`s as concise, end-user-readable transcript entries by default; unparseable events must not crash the UI.
- Provide a `/debug` toggle (discoverable in the command palette) to show redacted + truncated raw event payloads.
- Provide a structured status bar and improve transcript readability (dynamic header + padding).
