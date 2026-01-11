# Tasks

## 1. Core Infrastructure
- [x] 1.1 Add `id` field to `AgentConfig`
  - [x] Add `pub id: String` field to struct
  - [x] Update `Default::default()` to provide a default id
  - [x] Update all test constructors to include `id` field
  - [x] Update serialization tests to include `id` field
- [x] 1.2 Update `AgentRegistry::new()` to use stable IDs
  - [x] Change `default_id = default_agent.config().name.clone()` to `default_id = default_agent.config().id.clone()`
  - [x] Update tests that depend on name-based registry keys
  - [x] Add test verifying stable ID behavior

## 2. Agent Definitions
- [x] 2.1 Define built-in Plan agent configuration
  - [x] Create PLAN_AGENT constant with id="plan", name="Plan Agent"
  - [x] Set permissions: edit=Deny, bash=Deny, skill=Allow
  - [x] Set overrides: execute_command=Deny
  - [x] Add read-only instructions emphasizing analysis and research
  - [x] Define tool whitelist: read_file, glob, grep
- [x] 2.2 Define built-in Build agent configuration
  - [x] Create BUILD_AGENT constant with id="build", name="Build Agent"
  - [x] Set permissions: edit=Ask, bash=Ask, skill=Allow
  - [x] Add execution-oriented instructions
  - [x] Set default agent id to plan

## 3. Bootstrap Implementation
- [x] 3.1 Create `build_builtins()` function
  - [x] Change return type from `AgentComponents` to support multiple agents
  - [x] Create shared LLM provider configuration for both agents
  - [x] Create shared SandboxedPath for tools
  - [x] Instantiate Plan agent with Plan config
  - [x] Instantiate Build agent with Build config
  - [x] Return both agents in components struct
- [x] 3.2 Implement tool registration for Plan agent
  - [x] Add `register_read_only_tools()` helper to Agent
  - [x] Register only: read_file, glob, grep
  - [x] Verify no state-changing tools registered
  - [x] Add error handling for invalid tool names
- [x] 3.3 Implement tool registration for Build agent
  - [x] Register all tools: read_file, write_file, execute_command, glob, grep
  - [x] Verify full tool set available
- [x] 3.4 Update `build_agent_registry()` helper
  - [x] Extract default agent from BUILD_AGENT constant
  - [x] Register Plan agent with id="plan"
  - [x] Register Build agent as default
  - [x] Verify both agents in registry

## 4. Server Integration
- [x] 4.1 Update serve command
  - [x] Change from `build_agent()` to `build_builtins()`
  - [x] Use `build_agent_registry()` instead of direct registry construction
  - [x] Verify server starts with both agents registered
- [ ] 4.2 Update chat command (optional) - OUT OF SCOPE
  - [ ] Support agent selection in CLI mode
  - [ ] Default to plan agent for interactive sessions
  - **Note**: Per proposal Non-Goals, "Adding a CLI UX for agent selection beyond existing server/session agent_id selection" is out of scope. Server API already provides full agent selection capability via `agent_id` parameter.

## 5. Testing
- [x] 5.1 AgentConfig unit tests
  - [x] Test serialization with new `id` field
  - [x] Test deserialization with `id` and `name`
  - [x] Verify `id` and `name` are independent
- [x] 5.2 AgentRegistry tests
  - [x] Test registry uses `config.id` for lookups
  - [x] Test default agent ID is correct (plan)
  - [x] Test registering multiple agents with distinct IDs
  - [x] Test agent lookup by stable IDs (not names)
- [x] 5.3 Agent isolation tests
  - [x] Test Plan agent cannot call write_file (permission deny)
  - [x] Test Plan agent cannot call execute_command (permission deny)
  - [x] Test Plan agent has read_file, glob, grep available
  - [x] Test Build agent can call all tools (with Ask gating)
  - [x] Test Build agent has write_file, execute_command available
- [x] 5.4 Bootstrap integration tests
  - [x] Test Plan agent registers only read-only tools
  - [x] Test Build agent registers all tools
  - [x] Test both agents use the same model configuration
  - [x] Test default agent is Plan agent
- [x] 5.5 Server API tests
  - [x] Test `GET /api/v1/agents` returns both Plan and Build
  - [x] Test `GET /api/v1/agents/plan` returns Plan agent
  - [x] Test `GET /api/v1/agents/build` returns Build agent
  - [x] Test session creation with `agent_id: "plan"` uses Plan agent
  - [x] Test session creation with `agent_id: "build"` uses Build agent
  - [x] Test session creation without `agent_id` defaults to Plan agent
  - [x] Test agent switching mid-session works correctly
- [x] 5.6 Permission enforcement tests
  - [x] Test Plan agent permissions deny write operations
  - [x] Test Plan agent permissions deny bash operations
  - [x] Test Build agent permissions use Ask for dangerous operations
  - [x] Test permission overrides work correctly

## 6. Validation and Documentation
- [x] 6.1 Run comprehensive tests
  - [x] Run `cargo test` and ensure all tests pass
  - [x] Fix any failing tests
- [x] 6.2 Validate OpenSpec
  - [x] Run `openspec validate add-plan-build-builtin-agents --strict`
  - [x] Resolve all validation errors
