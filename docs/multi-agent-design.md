# Multi-Agent System Design: Plan & Build Agents

## Overview

This document describes the design for extending Sisyphus from a single built-in agent to two specialized built-in agents: **Plan** and **Build**. These agents serve distinct purposes—analysis/execution—while leveraging the existing multi-agent architecture.

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [Agent Roles & Responsibilities](#agent-roles--responsibilities)
3. [Configuration Structure](#configuration-structure)
4. [Bootstrap Changes](#bootstrap-changes)
5. [API Changes](#api-changes)
6. [Implementation Checklist](#implementation-checklist)
7. [Design Decisions](#design-decisions)
8. [Future Enhancements](#future-enhancements)

---

## Current Architecture Analysis

### Existing Components

| Component | Purpose | Current State |
|-----------|---------|---------------|
| `AgentConfig` | Defines agent metadata, permissions, system prompts | Supports multi-agent config |
| `AgentRegistry` | Manages multiple agents with registration and lookup | Ready for multiple agents |
| `Agent` | Runtime instance with tools, permissions, and LLM provider | Generic, reusable |
| Bootstrap System | Creates agents during startup | Currently creates only one agent |
| Session Management | Sessions have `agent_id` field for routing | Supports agent switching |
| Server API | `/api/v1/agents` endpoints | Already supports multi-agent |

### Key Finding

The architecture **already supports multiple agents**. The limitation is in the bootstrap process (`crates/cli/src/bootstrap.rs`), which only creates and registers one agent. The registry, session management, and API endpoints are all multi-agent ready.

---

## Agent Roles & Responsibilities

### Plan Agent

| Attribute | Value |
|-----------|-------|
| **ID** | `plan` |
| **Purpose** | Analysis, research, strategy formulation, code review, documentation |
| **Capabilities** | - Read files and analyze codebases<br>- Search and grep operations<br>- Provide architectural insights<br>- Suggest improvements<br>- Document findings |
| **Tools** | `read_file`, `glob`, `grep` (read-only tools only) |
| **Permissions** | - **Edit**: Deny<br>- **Bash**: Deny<br>- **Skill**: Allow |
| **Use Cases** | - "Analyze the architecture of this project"<br>- "Find all uses of function X"<br>- "Review the error handling patterns"<br>- "Document the authentication flow" |

### Build Agent

| Attribute | Value |
|-----------|-------|
| **ID** | `build` |
| **Purpose** | Task execution, file editing, command execution, external tool interaction |
| **Capabilities** | - Execute shell commands<br>- Read/write files<br>- Modify project structure<br>- Run tests and builds<br>- Interact with external APIs (via MCP) |
| **Tools** | All tools: `read_file`, `write_file`, `run_command`, `glob`, `grep` |
| **Permissions** | - **Edit**: Ask (configurable)<br>- **Bash**: Ask (configurable)<br>- **Skill**: Allow |
| **Use Cases** | - "Create a new API endpoint"<br>- "Fix the failing test"<br>- "Refactor this module"<br>- "Add error handling to function X" |
| **Default Agent** | ✅ Yes |

### Comparison

| Aspect | Plan Agent | Build Agent |
|--------|-----------|-------------|
| **Read Access** | ✅ Full | ✅ Full |
| **Write Access** | ❌ None | ✅ Full (with Ask gating) |
| **Command Execution** | ❌ None | ✅ Full (with Ask gating) |
| **Safety** | High (read-only) | Medium (Ask gating) |
| **Trust Model** | Always safe | Requires supervision |
| **Use in** | Exploration, research | Implementation, fixing |

---

## Configuration Structure

### Option A: Agent Definitions in Code (Recommended for v1)

**File:** `crates/core/src/agent/builtin.rs`

```rust
use crate::agent::config::{AgentConfig, AgentMode, AgentPermissions, PermissionLevel};

pub struct BuiltinAgentDefinition {
    pub id: String,
    pub config: AgentConfig,
    pub tool_whitelist: Option<Vec<String>>, // If None, all tools allowed
    pub is_default: bool,
}

/// Plan Agent: Read-only analysis and research
pub const PLAN_AGENT: BuiltinAgentDefinition = BuiltinAgentDefinition {
    id: "plan".to_string(),
    config: AgentConfig {
        name: "Plan Agent".to_string(),
        description: "Read-only agent for analysis, research, and strategy formulation".to_string(),
        instructions: r#"
You are a Planning Agent. Your role is to:

1. **Analyze codebases** and understand architecture patterns
2. **Provide research** and thorough documentation
3. **Suggest strategies** without making changes
4. **Ask clarifying questions** when requirements are ambiguous
5. **Identify patterns** and potential issues

You have READ-ONLY access to the workspace. Focus on:
- Thorough analysis and clear recommendations
- Understanding existing code structure
- Providing actionable insights
- Documenting findings comprehensively

When users ask you to "look into" something, provide:
- Summary of findings
- Relevant code locations
- Patterns observed
- Recommendations for next steps

Never attempt to modify files or execute commands.
        "#.to_string(),
        mode: AgentMode::Primary,
        permissions: AgentPermissions {
            edit: PermissionLevel::Deny,
            bash: PermissionLevel::Deny,
            skill: PermissionLevel::Allow,
            overrides: Default::default(),
        },
        command_path: None,
        context_limits: None,
        system_prompt_template: None,
    },
    tool_whitelist: Some(vec![
        "read_file".to_string(),
        "glob".to_string(),
        "grep".to_string(),
    ]),
    is_default: false,
};

/// Build Agent: Full execution and modification capabilities
pub const BUILD_AGENT: BuiltinAgentDefinition = BuiltinAgentDefinition {
    id: "build".to_string(),
    config: AgentConfig {
        name: "Build Agent".to_string(),
        description: "Executor agent for tasks, file modifications, and command execution".to_string(),
        instructions: r#"
You are a Build Agent. Your role is to:

1. **Execute commands** to build, test, and run projects
2. **Edit files** to implement solutions and fix issues
3. **Use tools efficiently** and responsibly
4. **Verify changes** work correctly before concluding
5. **Follow existing patterns** in the codebase

You have FULL access to modify the workspace. Be deliberate:
- Understand the task before making changes
- Use existing patterns and conventions
- Test your changes when appropriate
- Ask for approval before destructive operations

Best Practices:
- Read before writing (analyze existing code)
- Make minimal, focused changes
- Run tests after modifications
- Document significant changes
- Handle errors gracefully
        "#.to_string(),
        mode: AgentMode::Primary,
        permissions: AgentPermissions {
            edit: PermissionLevel::Ask,
            bash: PermissionLevel::Ask,
            skill: PermissionLevel::Allow,
            overrides: Default::default(),
        },
        command_path: None,
        context_limits: None,
        system_prompt_template: None,
    },
    tool_whitelist: None, // All tools allowed
    is_default: true,
};
```

**Pros:**
- Simpler to implement
- Type-safe
- Easy to test
- No external configuration files needed

**Cons:**
- Requires recompilation to change agent definitions
- Less flexible for power users

### Option B: Configuration File-Based (Future Enhancement)

**File:** `.sisyphus/agents.yaml`

```yaml
agents:
  - id: plan
    name: Plan Agent
    description: Read-only agent for analysis, research, and strategy formulation
    instructions: |
      You are a Planning Agent...
    mode: Primary
    permissions:
      edit: Deny
      bash: Deny
      skill: Allow
    tools:
      - read_file
      - glob
      - grep
    default: false

  - id: build
    name: Build Agent
    description: Executor agent for tasks, file modifications, and command execution
    instructions: |
      You are a Build Agent...
    mode: Primary
    permissions:
      edit: Ask
      bash: Ask
      skill: Allow
    default: true
```

**Pros:**
- User-configurable without recompilation
- Easier to add custom agents
- Supports multiple agent profiles

**Cons:**
- More complex implementation
- Validation required at runtime
- Potential for user misconfiguration

**Recommendation:** Start with **Option A** (code-based) for v1. Option B can be a v2 enhancement.

---

## Bootstrap Changes

### Current Implementation

**File:** `crates/cli/src/bootstrap.rs` (simplified)

```rust
pub struct AgentComponents {
    pub agent: Arc<Agent>,  // Only one agent
    pub bus: Arc<EventBus>,
}

pub async fn build_agent(config: &Config) -> anyhow::Result<AgentComponents> {
    // Creates a single agent with all tools
}
```

### Proposed Changes

**File:** `crates/cli/src/bootstrap.rs`

```rust
use common::{bus::EventBus, config::Config, llm::LLMProvider, logging, path::SandboxedPath};
use provider::{mock::MockProvider, openai::OpenAIProvider};
use sisyphus_core::agent::{config::AgentConfig, Agent};
use std::sync::Arc;
use tools::{
    cmd::CommandTool,
    fs::{ReadFileTool, WriteFileTool},
    glob::GlobTool,
    grep::GrepTool,
};

/// Updated: Now holds multiple agents
pub struct AgentComponents {
    pub agents: Vec<(String, Arc<Agent>)>,
    pub bus: Arc<EventBus>,
}

/// Updated: Creates all built-in agents
pub async fn build_builtins(config: &Config) -> anyhow::Result<AgentComponents> {
    let bus = Arc::new(EventBus::new(100));

    // Subscribe to bus for logging
    logging::start_event_logger(&bus).await;

    // Build LLM provider (shared across all agents)
    let provider: Box<dyn LLMProvider> = match config.llm.provider.as_str() {
        "mock" => Box::new(MockProvider::new()),
        _ => {
            println!(
                "Initializing provider: {} (model: {})",
                config.llm.provider, config.llm.model
            );
            if let Some(ref url) = config.llm.base_url {
                println!("Base URL: {}", url);
            } else if config.llm.provider != "openai" {
                println!(
                    "Warning: No base_url specified for custom provider. Defaulting to OpenAI."
                );
            }

            let api_key = config
                .llm
                .api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .expect("API Key must be set");
            Box::new(OpenAIProvider::new(
                api_key,
                config.llm.base_url.clone(),
                config.llm.model.clone(),
            ))
        }
    };

    let workspace_root = std::path::PathBuf::from(&config.workspace.root);
    let mut agents = Vec::new();

    // === CREATE PLAN AGENT ===
    let plan_config = sisyphus_core::agent::builtin::PLAN_AGENT.config.clone();
    let mut plan_agent = Agent::new(
        provider.clone(), // Share LLM provider
        bus.clone(),
        plan_config,
        workspace_root.clone(),
    );

    // Register Plan agent's read-only tools
    let cwd = std::env::current_dir()?;
    let sandbox = Arc::new(SandboxedPath::new(cwd)?);

    plan_agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    plan_agent.register_tool(Box::new(GlobTool::new(sandbox.clone())));
    plan_agent.register_tool(Box::new(GrepTool::new(sandbox.clone())));
    plan_agent.register_builtins(); // Help command

    agents.push((
        sisyphus_core::agent::builtin::PLAN_AGENT.id.clone(),
        Arc::new(plan_agent),
    ));

    // === CREATE BUILD AGENT ===
    let build_config = sisyphus_core::agent::builtin::BUILD_AGENT.config.clone();
    let mut build_agent = Agent::new(
        provider.clone(),
        bus.clone(),
        build_config,
        workspace_root.clone(),
    );

    // Register Build agent's full toolset
    build_agent.register_tool(Box::new(CommandTool));
    build_agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(WriteFileTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(GlobTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(GrepTool::new(sandbox)));
    build_agent.register_builtins();

    agents.push((
        sisyphus_core::agent::builtin::BUILD_AGENT.id.clone(),
        Arc::new(build_agent),
    ));

    Ok(AgentComponents { agents, bus })
}

/// New: Creates registry from built-in agents
pub fn build_agent_registry(components: AgentComponents) -> Arc<AgentRegistry> {
    // Determine default agent (Build agent is marked as default)
    let default_agent = components
        .agents
        .iter()
        .find(|(_, agent)| agent.config().name.contains("Build"))
        .map(|(_, agent)| agent.clone())
        .unwrap_or_else(|| components.agents[0].1.clone());

    let mut registry = AgentRegistry::new(default_agent);

    // Register all non-default agents
    for (id, agent) in components.agents {
        if agent.config().name != registry.get_default_agent().config().name {
            registry.register(id, agent);
        }
    }

    Arc::new(registry)
}
```

### File Structure

```
crates/
├── core/
│   └── src/
│       └── agent/
│           ├── builtin.rs          # NEW: Plan/Build agent definitions
│           ├── config.rs
│           ├── prompt.rs
│           ├── registry.rs
│           └── agent.rs
└── cli/
    └── src/
        └── bootstrap.rs           # UPDATED: Creates multiple agents
```

---

## API Changes

### Server Command

**File:** `crates/cli/src/commands/serve.rs`

```rust
use crate::bootstrap;
use common::config::Config;
use rust_i18n::t;
use sisyphus_core::session::manager::SessionManager;
use std::sync::Arc;

pub async fn run(config: Config, port: u16) -> anyhow::Result<()> {
    println!("{}", t!("starting_server", port = port));

    // Build all built-in agents (CHANGED)
    let components = bootstrap::build_builtins(&config).await?;

    // Create registry from all agents (CHANGED)
    let registry = bootstrap::build_agent_registry(components.clone());

    let session_manager = Arc::new(SessionManager::new());

    server::Server::new(port, registry, session_manager, components.bus)
        .run()
        .await?;

    Ok(())
}
```

### Chat Command (Optional Enhancement)

**File:** `crates/cli/src/commands/chat.rs`

```rust
pub async fn run(config: Config, config_path: Option<String>, tui: bool) -> anyhow::Result<()> {
    let components = bootstrap::build_builtins(&config).await?;
    let registry = bootstrap::build_agent_registry(components.clone());

    let session_manager = Arc::new(SessionManager::new());

    // Default to build agent for interactive sessions
    let session_lock = session_manager.create_session(Some("build".to_string()));
    let session_id = session_lock.read().await.id.clone();

    // ... rest of chat logic

    Ok(())
}
```

### Existing API Endpoints (No Changes Needed)

The following endpoints already support multi-agent operations:

#### List Available Agents
```
GET /api/v1/agents
```

**Response:**
```json
[
  {
    "id": "build",
    "model": "gpt-4",
    "name": "Build Agent",
    "description": "Executor agent for tasks, file modifications, and command execution"
  },
  {
    "id": "plan",
    "model": "gpt-4",
    "name": "Plan Agent",
    "description": "Read-only agent for analysis, research, and strategy formulation"
  }
]
```

#### Get Agent Details
```
GET /api/v1/agents/:id
```

#### Create Session with Specific Agent
```
POST /api/v1/sessions
```

**Request:**
```json
{
  "agent_id": "plan"  // or "build", or omit for default
}
```

#### Switch Agent Mid-Session
```
PUT /api/v1/sessions/:id/agent
```

**Request:**
```json
{
  "agent_id": "build"
}
```

#### Chat with Session Agent
```
POST /api/v1/sessions/:id/chat
```

**Response includes:**
```json
{
  "response": "...",
  "session_id": "abc123",
  "usage": "1234 tokens",
  "model": "gpt-4",
  "agent_id": "build"
}
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure

- [ ] Create `crates/core/src/agent/builtin.rs` module
  - [ ] Define `BuiltinAgentDefinition` struct
  - [ ] Define `PLAN_AGENT` constant with complete configuration
  - [ ] Define `BUILD_AGENT` constant with complete configuration
  - [ ] Export in `crates/core/src/agent/mod.rs`

- [ ] Update `crates/cli/src/bootstrap.rs`
  - [ ] Change `AgentComponents` to hold `Vec<(String, Arc<Agent>)>`
  - [ ] Rename `build_agent()` → `build_builtins()`
  - [ ] Implement Plan agent creation with read-only tools
  - [ ] Implement Build agent creation with full toolset
  - [ ] Add `build_agent_registry()` helper function
  - [ ] Update all call sites

- [ ] Update `crates/cli/src/commands/serve.rs`
  - [ ] Use new `build_builtins()` function
  - [ ] Use new `build_agent_registry()` function
  - [ ] Verify server starts with both agents registered

### Phase 2: Testing

- [ ] Add unit tests for `builtin.rs`
  - [ ] Test Plan agent configuration
  - [ ] Test Build agent configuration
  - [ ] Test tool whitelist logic

- [ ] Add integration tests for bootstrap
  - [ ] Test Plan agent has only read-only tools
  - [ ] Test Build agent has full toolset
  - [ ] Test both agents use same LLM provider
  - [ ] Test Build agent is registered as default

- [ ] Add server API tests
  - [ ] Test `GET /api/v1/agents` returns both agents
  - [ ] Test `GET /api/v1/agents/:id` for each agent
  - [ ] Test session creation with specific agent
  - [ ] Test agent switching mid-session
  - [ ] Verify Plan agent cannot execute tools it doesn't have

### Phase 3: Documentation

- [ ] Update README.md
  - [ ] Document Plan vs Build agent distinction
  - [ ] Provide usage examples for each agent
  - [ ] Explain agent selection workflow

- [ ] Update API documentation
  - [ ] Document `/api/v1/agents` endpoints
  - [ ] Document agent selection in session creation
  - [ ] Provide example requests/responses

- [ ] Create examples
  - [ ] Example: Using Plan agent for code analysis
  - [ ] Example: Using Build agent for feature implementation
  - [ ] Example: Switching between agents

### Phase 4: Optional Enhancements

- [ ] CLI agent selection
  - [ ] Add `--agent` flag to chat command
  - [ ] Add interactive agent selection prompt
  - [ ] Display current agent in chat UI

- [ ] Custom agent templates
  - [ ] Support `.sisyphus/templates/plan_agent.jinja`
  - [ ] Support `.sisyphus/templates/build_agent.jinja`
  - [ ] Load agent-specific templates at startup

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Default Agent** | Build | Most users want to execute tasks, not just analyze. Build agent can also read files. |
| **Tool Filtering** | Registration-time | Clean separation of concerns, no runtime overhead, impossible to circumvent |
| **Permissions** | Config-based | Already supported in `AgentConfig`, enforces safety boundaries |
| **LLM Provider** | Shared across agents | Reduces cost, simpler architecture, easier debugging |
| **Configuration Method** | Code-based (v1) | Faster to implement, easier to test, type-safe |
| **Agent Switching** | Session-level | Already supported in existing API, no changes needed |
| **Agent Instructions** | Hardcoded in constants | Ensures consistent behavior, no external file dependency |
| **Bus Sharing** | Single EventBus | Enables cross-agent event tracking and debugging |

### Trade-offs

| Aspect | Consideration | Decision |
|--------|---------------|----------|
| **Code complexity** | Adding two agents increases bootstrap complexity | Acceptable - bootstrap is isolated, well-tested |
| **Flexibility** | Code-based is less flexible than config | Acceptable for v1 - prioritize stability and correctness |
| **User confusion** | Two agents might confuse users | Mitigate with clear documentation and default to Build |
| **Testing overhead** | Need to test both agents | Acceptable - most tests can be parameterized |
| **Runtime overhead** | Two agents = more memory | Negligible - shared LLM provider, tools are lightweight |

---

## Future Enhancements

### Out of Scope for v1

1. **Dynamic Agent Loading**
   - Load agents from `.sisyphus/agents.yaml`
   - Add/remove agents at runtime
   - Support custom user-defined agents

2. **Agent Auto-Selection**
   - Analyze user request and suggest appropriate agent
   - Automatic agent switching based on task type
   - ML-based agent recommendation

3. **Sub-Agent Delegation**
   - Plan agent can delegate to Build agent
   - Build agent can spawn specialized sub-agents
   - Multi-agent workflow orchestration

4. **Agent Capabilities API**
   - `GET /api/v1/agents/:id/capabilities` endpoint
   - List available tools per agent
   - Introspect permission levels

5. **Per-Agent Context Limits**
   - Different token budgets for each agent
   - Custom context compaction strategies
   - Agent-specific pinned messages

6. **Agent Profiles**
   - Multiple Plan/Build configurations
   - User can create profiles (e.g., "Plan-Critical", "Plan-Quick")
   - Profile selection at session creation

7. **Agent Telemetry**
   - Track agent usage patterns
   - Measure success rates per agent
   - Optimize agent selection based on history

### Potential v2 Features

1. **Configuration File-Based Agents**
   - Implement Option B from [Configuration Structure](#configuration-structure)
   - Validation layer for agent definitions
   - Migration tool from v1 to v2 config

2. **Skill Integration**
   - Agent-specific skill loading
   - Different skill sets for Plan vs Build
   - Skill recommendations per agent

3. **Agent Collaboration**
   - Plan agent can request Build agent to execute
   - Build agent can ask Plan agent for analysis
   - Multi-agent conversation chains

4. **Custom Tool Sets**
   - User-defined tool groups per agent
   - Tool dependencies and prerequisites
   - Dynamic tool registration at runtime

---

## Example Workflows

### Workflow 1: Code Analysis (Plan Agent)

```bash
# Create session with Plan agent
curl -X POST http://localhost:3000/api/v1/sessions \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "plan"}'

# Send analysis request
curl -X POST http://localhost:3000/api/v1/sessions/abc123/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Analyze the authentication flow in this project"}'

# Result: Plan agent reads files, provides analysis
# No risk of accidental file modifications
```

### Workflow 2: Feature Implementation (Build Agent)

```bash
# Create session with Build agent (default)
curl -X POST http://localhost:3000/api/v1/sessions \
  -H "Content-Type: application/json"

# Send implementation request
curl -X POST http://localhost:3000/api/v1/sessions/abc123/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Add rate limiting to the API endpoints"}'

# Result: Build agent reads code, edits files, runs tests
# Permissions gate destructive operations
```

### Workflow 3: Hybrid (Switch Agents)

```bash
# Start with Plan agent for research
curl -X POST http://localhost:3000/api/v1/sessions/abc123/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Find all files that handle user authentication"}'

# Switch to Build agent for implementation
curl -X PUT http://localhost:3000/api/v1/sessions/abc123/agent \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "build"}'

# Implement changes
curl -X POST http://localhost:3000/api/v1/sessions/abc123/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Add OAuth2 support to the authentication system"}'
```

### Workflow 4: CLI Usage

```bash
# Start chat with Build agent (default)
sisyphus chat

# Inside chat:
> Analyze the project structure
# (Uses Plan agent automatically if we implement auto-selection)
> Implement caching for the API
# (Uses Build agent)

# Or explicit selection:
> /agent switch plan
> Review the error handling patterns

> /agent switch build
> Add proper error logging
```

---

## Testing Strategy

### Unit Tests

**File:** `crates/core/tests/agent_builtin_test.rs`

```rust
#[cfg(test)]
mod tests {
    use sisyphus_core::agent::builtin::{PLAN_AGENT, BUILD_AGENT};

    #[test]
    fn test_plan_agent_config() {
        assert_eq!(PLAN_AGENT.id, "plan");
        assert_eq!(PLAN_AGENT.config.name, "Plan Agent");
        assert_eq!(
            PLAN_AGENT.config.permissions.edit,
            PermissionLevel::Deny
        );
    }

    #[test]
    fn test_build_agent_config() {
        assert_eq!(BUILD_AGENT.id, "build");
        assert_eq!(BUILD_AGENT.config.name, "Build Agent");
        assert_eq!(
            BUILD_AGENT.config.permissions.edit,
            PermissionLevel::Ask
        );
    }

    #[test]
    fn test_tool_whitelist() {
        assert!(PLAN_AGENT.tool_whitelist.is_some());
        assert!(BUILD_AGENT.tool_whitelist.is_none());
    }
}
```

### Integration Tests

**File:** `crates/server/tests/multi_agent_test.rs`

```rust
#[tokio::test]
async fn test_list_multiple_agents() {
    // Test that /api/v1/agents returns both Plan and Build agents
    let response = client.get("/api/v1/agents").send().await.unwrap();
    let agents: Vec<AgentResponse> = response.json().await.unwrap();

    assert_eq!(agents.len(), 2);
    assert!(agents.iter().any(|a| a.id == "plan"));
    assert!(agents.iter().any(|a| a.id == "build"));
}

#[tokio::test]
async fn test_plan_agent_cannot_execute_commands() {
    // Create session with Plan agent
    // Try to execute a command
    // Verify it fails or is denied
}

#[tokio::test]
async fn test_build_agent_can_execute_commands() {
    // Create session with Build agent
    // Execute a command
    // Verify it succeeds (or prompts for approval)
}
```

---

## Security Considerations

### Plan Agent Security

- **Read-only by design**: Cannot modify files or execute commands
- **Tool whitelist**: Only tools with read capabilities are registered
- **Permission hardening**: `edit` and `bash` set to `Deny` in config
- **Attack surface**: Minimal - limited to information disclosure via tool outputs

### Build Agent Security

- **Ask-gating**: Destructive operations require user approval
- **Command audit**: All tool executions are logged via EventBus
- **Sandboxing**: File operations use `SandboxedPath` for boundary enforcement
- **Session isolation**: Each session has independent state and permissions

### Cross-Agent Security

- **No shared state**: Agents cannot access each other's memory or context
- **Event bus logging**: All agent actions are auditable
- **Session-level isolation**: Switching agents doesn't leak context between them
- **Permission enforcement**: Permissions are checked at tool execution time, not registration

---

## Performance Considerations

### Memory Usage

- **Shared LLM Provider**: Both agents share the same provider instance
- **Tool Registration**: Tools are registered once per agent, lightweight
- **Session Memory**: Each session maintains its own message history
- **Event Bus**: Single bus for all agents, minimal overhead

### Response Time

- **Agent Lookup**: O(1) HashMap lookup in `AgentRegistry`
- **Tool Execution**: No overhead vs single agent design
- **Context Building**: Same as single agent, just different system prompts
- **Permission Checks**: O(1) hashmap lookup for permission level

### Scalability

- **Multiple Sessions**: Each session has independent agent assignment
- **Concurrent Requests**: Axum handles concurrent chat requests naturally
- **Agent Registry**: Thread-safe via `RwLock`, supports concurrent reads
- **No Bottlenecks**: Shared components (provider, bus) are designed for concurrency

---

## Migration Path

### From Single Agent to Multi-Agent

**Existing Behavior:**
- Single agent with all tools
- Permissions configurable via config file

**New Behavior:**
- Two agents with different tool sets and permissions
- Same API endpoints, but now returns multiple agents
- Existing sessions continue to work (backward compatible)

**Breaking Changes:**
- None. Existing code that creates a session without specifying `agent_id` will get the Build agent (new default).

**Migration Steps:**
1. Implement multi-agent bootstrap (Phase 1)
2. Add tests (Phase 2)
3. Update documentation (Phase 3)
4. Release v1.0 with multi-agent support
5. Optional: Add CLI agent selection (Phase 4)

---

## Glossary

| Term | Definition |
|------|------------|
| **Agent** | An instance of the `Agent` struct with specific configuration, tools, and permissions |
| **Built-in Agent** | Pre-configured agents (Plan, Build) that are always available |
| **Tool** | A capability (e.g., `read_file`, `run_command`) that agents can use |
| **Permission Level** | `Allow`, `Ask`, `Deny` - controls how tool execution is gated |
| **Session** | A conversation context with associated agent and message history |
| **AgentRegistry** | Central registry managing all available agents |
| **Bootstrap** | Startup process that initializes agents and services |
| **EventBus** | Pub/sub system for distributing system events (e.g., tool execution) |
| **SandboxedPath** | File system boundary enforcement for tool operations |

---

## References

- [PRD.md](../PRD.md) - Product Requirements Document
- [AGENTS.md](../AGENTS.md) - Agent Development Guidelines
- [crates/core/src/agent.rs](../crates/core/src/agent.rs) - Agent implementation
- [crates/core/src/agent/config.rs](../crates/core/src/agent/config.rs) - Agent configuration
- [crates/core/src/agent/registry.rs](../crates/core/src/agent/registry.rs) - Agent registry
- [crates/server/src/lib.rs](../crates/server/src/lib.rs) - Server API implementation

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-01-10 | Sisyphus | Initial design document |
