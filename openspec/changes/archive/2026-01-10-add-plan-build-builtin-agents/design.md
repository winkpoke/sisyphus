# Design: Plan and Build built-in agents

## Current Architecture Analysis
The system already contains the multi-agent primitives needed to support multiple agents:
- Agent configuration and prompt construction are per-Agent.
- A registry exists for multiple Agents.
- Sessions store `agent_id` for routing.
- The server exposes Agent discovery endpoints.

The primary gap is runtime bootstrap: default server startup currently constructs only one Agent instance and registers only that Agent.

## Overview
Introduce two built-in agents with distinct responsibilities:
- `plan`: read-only planning and analysis
- `build`: implementation and execution

Both agents share the same core runtime (prompt templating, session integration, tool-call loop) but differ in:
- Tool exposure (which tools are registered on the Agent)
- Permission defaults (Allow/Ask/Deny per category and per-tool overrides)
- Instruction text (prompt instructions emphasizing planning vs execution)

## Agent Identity
Agent identity is split into two concepts:
- `agent_id`: a stable routing identifier used by sessions and server API (e.g., `plan`, `build`)
- `name`: a display label (e.g., `Plan Agent`, `Build Agent`)

**Rationale:** Server routes turns and exposes discovery endpoints keyed by `agent_id`. A stable ID avoids breaking clients if display names change.

## Default Selection
The default agent id is `plan`.

**Rationale:**
- Safe-by-default: new sessions start in a non-mutating posture
- Users can explicitly switch to `build` when they intend to execute
- Prevents accidental command execution or file writes in unaware clients

**Server behavior:**
- If a session has no `agent_id`, it is treated as `plan`.
- If a client specifies `agent_id`, the server validates it exists in the registry.

## Tool Exposure and Permissions
Defense-in-depth is achieved via two layers:

**1. Tool exposure:**
- `plan` registers only read-only tools (e.g., `read_file`, `glob`, `grep`).
- `build` registers the full default set (including `execute_command` and `write_file`).

**2. Permission policy:**
- `plan` denies state-changing tools even if they become registered accidentally.
- `build` keeps dangerous tools Ask-gated by default.

Due to current permission categorization based on tool name, `plan` SHOULD deny `execute_command` via per-tool overrides to preserve read-only behavior even if that tool is registered.

This design ensures the Plan agent cannot mutate the workspace or execute commands, while still being able to inspect the repository to produce grounded plans.

## Configuration Strategy
**v1: Code-defined built-ins:**
Define two built-in agents in code as defaults (id, display metadata, prompt instructions, permissions) and register them at startup.

**Future enhancement: File-defined agents**
Allow overriding or extending built-ins via a workspace config file. This is out of scope for this change.

## Provider and Model
Both agents are initialized with provider instances derived from the same runtime configuration (same base_url/model/api key unless explicitly extended later). This keeps the initial change small while preserving a clear path to per-agent model configuration in the future.

Per-agent provider sharing is out of scope for v1 of this change because the current `Agent` type owns a `Box<dyn LLMProvider>`. A future change can refactor the agent/provider boundary to enable shared providers.

## Tool Duplication Tradeoff
**Decision:** Accept tool instance duplication for v1.

**Rationale:**
- Tools are lightweight (few KB each)
- Memory overhead is negligible (<1MB for all tools)
- Simpler implementation is better than premature optimization
- Sharing tools would require `Arc<dyn Tool>` wrapper, adding complexity

**Future enhancement:** If profiling shows memory issues, revisit with shared tool instances.

## Implementation Notes
- Agent registry ids MUST be explicit and stable (`plan`, `build`) and MUST NOT depend on the display name.
- Plan safety SHOULD be enforced by both (a) not registering state-changing tools and (b) permission denies for known state-changing tools.

## AgentConfig Changes
**CRITICAL:** This change requires adding an `id` field to `AgentConfig`.

Currently, `AgentConfig` only has a `name` field:
```rust
pub struct AgentConfig {
    pub name: String,  // Currently used as both display name AND registry key
    // ...
}
```

This creates a problem: the registry key is derived from `name` (e.g., "Plan Agent"), but the server API expects stable IDs like `"plan"` or `"build"`.

**Required change:**
```rust
pub struct AgentConfig {
    pub id: String,        // NEW: Stable routing identifier (e.g., "plan", "build")
    pub name: String,      // Display name (e.g., "Plan Agent", "Build Agent")
    // ...
}
```

**Impact:**
- `AgentRegistry::new()` must use `config.id` instead of `config.name`
- All AgentConfig construction must include the `id` field
- Server API lookups will use stable IDs instead of display names

## Tool Validation Strategy
**Rationale:** To prevent accidental registration of the wrong tools, add runtime validation.

**Approach 1 (Recommended for v1): Documented Safety**
- Add a helper function `register_read_only_tools()` that only allows specific tool names
- Use this for Plan agent bootstrap
- Keep `register_tool()` as-is for simplicity

```rust
impl Agent {
    pub fn register_read_only_tools(&mut self, tools: Vec<Box<dyn Tool>>) -> Result<()> {
        for tool in tools {
            match tool.name() {
                "read_file" | "glob" | "grep" => {
                    self.register_tool(tool);
                }
                _ => return Err(anyhow!("Tool '{}' not allowed in read-only agent", tool.name())),
            }
        }
        Ok(())
    }
}
```

**Approach 2 (Future v2): Whitelist in AgentConfig**
- Add `allowed_tools: Option<Vec<String>>` to `AgentConfig`
- Check at registration time against this list
- More flexible but adds complexity

**Decision:** Use Approach 1 for v1 - explicit, simple, no config changes.

## Migration Path
**Impact on existing clients:** This change is backward compatible with the following caveats:

**What changes:**
1. `/api/v1/agents` now returns 2 agents instead of 1
2. New sessions without an explicit `agent_id` default to `"plan"` (changed from single agent behavior)
3. Agent IDs change from display names to stable IDs

**Client update recommendations:**
- Update agent selection UI to show both Plan and Build options
- Default to the Plan agent for safe-by-default workflows
- Use Plan for analysis/research; switch to Build for execution
- Add agent switching capability to the UI

**Breaking changes:**
- None, assuming clients use the API correctly (they should already handle multiple agents since the system supports it)

**Versioning:** No API version bump required - this is feature expansion, not a breaking change.
