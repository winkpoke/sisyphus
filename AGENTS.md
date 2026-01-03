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
- Clients render permission prompts (operation/tool_name/call_id) and can submit approve/deny decisions to resume.
