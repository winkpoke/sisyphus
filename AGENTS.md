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
- **Command Handling**: Slash commands and their effects (e.g., `/new`, `/exit`) are processed by the core `ChatService` to ensure deterministic session state management across all clients.
- **Multi-Agent Routing**: Server supports agent discovery (`/api/v1/agents`) and per-session agent assignment (`POST/PUT /api/v1/sessions`). Chat requests are routed to the session's active agent.
- **Event Bus**: Uses a typed, topic-based distribution system. Components must subscribe using `subscribe_raw()` for global auditing or specific topics for efficiency.
- **LLM Providers must use the shared `SSEParser` (`crates/provider/src/sse.rs`) for streaming to ensure correct handling of split network chunks and multi-byte characters.

CLI TUI requirements (see `spec/cli-tui` in OpenSpec):
- **Architecture**: Follow the Model-View-Update (MVU) pattern with a pure `update` function and centralized `Action` enum.
- **Visuals**: Use the centralized `Theme` struct for semantic colors; avoid hardcoded ANSI values.
- **Feedback**: Use `Toast` overlays for transient user feedback (e.g., clipboard success) and spinners for active states.
- **Events**: Render backend SSE `SystemEvent`s as concise, end-user-readable transcript entries by default; unparseable events must not crash the UI.
- **Debug**: Provide a `/debug` toggle (discoverable in the command palette) to show redacted + truncated raw event payloads.
- **Layout**: Provide a structured status bar and improve transcript readability (dynamic header + padding).
