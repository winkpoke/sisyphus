# Tasks

## 1. Core Infrastructure
- [ ] 1.1 Add `id` field to `AgentConfig`
  - [ ] Add `pub id: String` field to struct
  - [ ] Update `Default::default()` to provide a default id
  - [ ] Update all test constructors to include `id` field
  - [ ] Update serialization tests to include `id` field
- [ ] 1.2 Update `AgentRegistry::new()` to use stable IDs
  - [ ] Change `default_id = default_agent.config().name.clone()` to `default_id = default_agent.config().id.clone()`
  - [ ] Update tests that depend on name-based registry keys
  - [ ] Add test verifying stable ID behavior

## 2. Agent Definitions
- [ ] 2.1 Define built-in Plan agent configuration
  - [ ] Create PLAN_AGENT constant with id="plan", name="Plan Agent"
  - [ ] Set permissions: edit=Deny, bash=Deny, skill=Allow
  - [ ] Set overrides: execute_command=Deny
  - [ ] Add read-only instructions emphasizing analysis and research
  - [ ] Define tool whitelist: read_file, glob, grep
- [ ] 2.2 Define built-in Build agent configuration
  - [ ] Create BUILD_AGENT constant with id="build", name="Build Agent"
  - [ ] Set permissions: edit=Ask, bash=Ask, skill=Allow
  - [ ] Add execution-oriented instructions
  - [ ] Set default agent id to plan

## 3. Bootstrap Implementation
- [ ] 3.1 Create `build_builtins()` function
  - [ ] Change return type from `AgentComponents` to support multiple agents
  - [ ] Create shared LLM provider configuration for both agents
  - [ ] Create shared SandboxedPath for tools
  - [ ] Instantiate Plan agent with Plan config
  - [ ] Instantiate Build agent with Build config
  - [ ] Return both agents in components struct
- [ ] 3.2 Implement tool registration for Plan agent
  - [ ] Add `register_read_only_tools()` helper to Agent
  - [ ] Register only: read_file, glob, grep
  - [ ] Verify no state-changing tools registered
  - [ ] Add error handling for invalid tool names
- [ ] 3.3 Implement tool registration for Build agent
  - [ ] Register all tools: read_file, write_file, execute_command, glob, grep
  - [ ] Verify full tool set available
- [ ] 3.4 Update `build_agent_registry()` helper
  - [ ] Extract default agent from BUILD_AGENT constant
  - [ ] Register Plan agent with id="plan"
  - [ ] Register Build agent as default
  - [ ] Verify both agents in registry

## 4. Server Integration
- [ ] 4.1 Update serve command
  - [ ] Change from `build_agent()` to `build_builtins()`
  - [ ] Use `build_agent_registry()` instead of direct registry construction
  - [ ] Verify server starts with both agents registered
- [ ] 4.2 Update chat command (optional)
  - [ ] Support agent selection in CLI mode
  - [ ] Default to plan agent for interactive sessions

## 5. Testing
- [ ] 5.1 AgentConfig unit tests
  - [ ] Test serialization with new `id` field
  - [ ] Test deserialization with `id` and `name`
  - [ ] Verify `id` and `name` are independent
- [ ] 5.2 AgentRegistry tests
  - [ ] Test registry uses `config.id` for lookups
  - [ ] Test default agent ID is correct (plan)
  - [ ] Test registering multiple agents with distinct IDs
  - [ ] Test agent lookup by stable IDs (not names)
- [ ] 5.3 Agent isolation tests
  - [ ] Test Plan agent cannot call write_file (permission deny)
  - [ ] Test Plan agent cannot call execute_command (permission deny)
  - [ ] Test Plan agent has read_file, glob, grep available
  - [ ] Test Build agent can call all tools (with Ask gating)
  - [ ] Test Build agent has write_file, execute_command available
- [ ] 5.4 Bootstrap integration tests
  - [ ] Test Plan agent registers only read-only tools
  - [ ] Test Build agent registers all tools
  - [ ] Test both agents use the same model configuration
  - [ ] Test default agent is Plan agent
- [ ] 5.5 Server API tests
  - [ ] Test `GET /api/v1/agents` returns both Plan and Build
  - [ ] Test `GET /api/v1/agents/plan` returns Plan agent
  - [ ] Test `GET /api/v1/agents/build` returns Build agent
  - [ ] Test session creation with `agent_id: "plan"` uses Plan agent
  - [ ] Test session creation with `agent_id: "build"` uses Build agent
  - [ ] Test session creation without `agent_id` defaults to Build agent
  - [ ] Test agent switching mid-session works correctly
- [ ] 5.6 Permission enforcement tests
  - [ ] Test Plan agent permissions deny write operations
  - [ ] Test Plan agent permissions deny bash operations
  - [ ] Test Build agent permissions use Ask for dangerous operations
  - [ ] Test permission overrides work correctly

## 6. Validation and Documentation
- [ ] 6.1 Run comprehensive tests
  - [ ] Run `cargo test` and ensure all tests pass
  - [ ] Fix any failing tests
- [ ] 6.2 Validate OpenSpec
  - [ ] Run `openspec validate add-plan-build-builtin-agents --strict`
  - [ ] Resolve all validation errors
