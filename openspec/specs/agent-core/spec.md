# agent-core Specification

## Purpose
Defines the core architecture for Sisyphus Agents, including configuration, permissions, agent modes, and dynamic prompt generation. Agents are stateless execution engines that process chat turns, manage tool execution with permission gating, and expand SlashCommands via Jinja2 templates.
## Requirements
### Requirement: Agent Mode
The system SHALL support agent modes that control behavior and scope of operations.

#### Scenario: Agent mode enumeration
- **GIVEN** the Agent configuration is loaded
- **THEN** the mode field SHALL be one of: Primary, SubAgent, or All
- **AND** mode SHALL default to Primary

#### Scenario: Primary agent mode
- **GIVEN** an agent is configured with mode Primary
- **WHEN** the agent executes tool calls
- **THEN** the agent SHALL check the tool against permissions.allow, permissions.deny, and permissions.ask lists
- **AND** the agent SHALL require approval if the tool has Ask permission level

### Requirement: Permission Modes

The system SHALL support five permission modes that define default behavior when no explicit rule matches: Default, AcceptEdits, DontAsk, BypassPermissions, and Plan.

#### Scenario: Default mode prompts for all non-allowed operations

- **GIVEN** permission mode is `Default`
- **AND** no allow/ask/deny rules are configured
- **WHEN** agent attempts to execute any tool
- **THEN** system emits a permission request (Ask behavior)
- **AND** system waits for user approval before execution

#### Scenario: AcceptEdits mode auto-approves Write and Edit operations

- **GIVEN** permission mode is `AcceptEdits`
- **AND** no allow/ask/deny rules are configured
- **WHEN** agent attempts to execute `write_file` tool
- **THEN** tool executes immediately without prompting
- **WHEN** agent attempts to execute `replace_in_file` tool
- **THEN** tool executes immediately without prompting
- **WHEN** agent attempts to execute `run_command` tool
- **THEN** system emits a permission request (Ask behavior)

#### Scenario: DontAsk mode auto-denies all operations except explicitly allowed

- **GIVEN** permission mode is `DontAsk`
- **AND** allow list contains `"Bash(git:*)"`
- **WHEN** agent attempts to execute `git diff` command
- **THEN** command executes immediately without prompting
- **WHEN** agent attempts to execute `npm install` command
- **THEN** tool is denied without prompting
- **AND** system returns "Permission denied: tool execution requires explicit allow rule."

#### Scenario: BypassPermissions mode skips permission checks except deny rules

- **GIVEN** permission mode is `BypassPermissions`
- **AND** deny list contains `"Read(./.env)"`
- **WHEN** agent attempts to execute `read_file` on `.env`
- **THEN** tool is denied (deny rules are always enforced, even in bypass mode)
- **WHEN** agent attempts to execute `write_file` on `./src/main.rs` (no deny rule matches)
- **THEN** tool executes immediately without permission checking (allow/ask checks and mode fallback are skipped)
- **AND** bypass mode is documented as dangerous because non-denied operations execute without prompting

#### Scenario: Plan mode denies all Write and Edit operations

- **GIVEN** permission mode is `Plan`
- **AND** no allow/ask/deny rules are configured
- **WHEN** agent attempts to execute `write_file` tool
- **THEN** tool is denied without prompting
- **AND** system returns "Permission denied: Plan mode does not allow write operations."
- **WHEN** agent attempts to execute `read_file` tool
- **THEN** system emits a permission request (Ask behavior for read-only tools)

### Requirement: Rule-Based Permission System

The system SHALL support rule-based permission lists (allow, ask, deny) that match tools and their arguments using pattern matching, with deny rules taking precedence over allow/ask rules.

#### Scenario: Deny rules take precedence over allow rules

- **GIVEN** deny list contains `"Bash(rm -rf:*)"`
- **AND** allow list contains `"Bash(rm:*)"`
- **WHEN** agent attempts to execute `rm -rf /tmp`
- **THEN** tool is denied (deny rule wins)
- **AND** system does not check allow rules after deny match

#### Scenario: Bash commands use prefix matching

- **GIVEN** allow list contains `"Bash(git:*)"`
- **WHEN** agent attempts to execute `git push origin main`
- **THEN** command executes immediately (prefix match)
- **WHEN** agent attempts to execute `npm install`
- **THEN** system emits permission request (no match)

#### Scenario: File operations use glob patterns

- **GIVEN** allow list contains `"Read(./src/**/*.rs)"`
- **WHEN** agent attempts to read `./src/agent/config.rs`
- **THEN** file is read immediately (glob match)
- **WHEN** agent attempts to read `./docs/README.md`
- **THEN** system emits permission request (no match)

#### Scenario: Exact tool name matching for non-file tools

- **GIVEN** deny list contains `"WebFetch"`
- **WHEN** agent attempts to execute `WebFetch` tool with any arguments
- **THEN** tool is denied (exact name match)
- **AND** pattern matching is not applied

#### Scenario: Wildcard rule allows all operations for a tool

- **GIVEN** allow list contains `"Bash(*)"` or `"Bash()"`
- **WHEN** agent attempts to execute any Bash command
- **THEN** command executes immediately (wildcard match)
- **AND** no permission check is performed for Bash tool

### Requirement: Permission Resolution Order

The system SHALL resolve permissions using a deterministic order: check deny rules, then allow rules, then ask rules, and finally fall back to permission mode behavior.

#### Scenario: Deny rule prevents execution before other checks

- **GIVEN** deny list contains `"Read(./.env)"`
- **AND** allow list contains `"Read(./.env)"`
- **AND** mode is `BypassPermissions`
- **WHEN** agent attempts to read `.env` file
- **THEN** tool is denied (deny checked first, short-circuits)
- **AND** allow rules are not evaluated
- **AND** mode is not considered

#### Scenario: Allow rule executes without prompting

- **GIVEN** allow list contains `"Write(./src/**)"`
- **AND** mode is `Default`
- **AND** no deny rule matches
- **WHEN** agent attempts to write `./src/main.rs`
- **THEN** tool executes immediately without prompting
- **AND** ask rules are not evaluated
- **AND** mode fallback is not triggered

#### Scenario: Ask rule prompts even if mode would allow

- **GIVEN** ask list contains `"Bash(npm install:*)"`
- **AND** mode is `AcceptEdits`
- **WHEN** agent attempts to execute `npm install lodash`
- **THEN** system emits permission request (ask rule matches)
- **AND** mode is not evaluated

#### Scenario: No rule match triggers permission mode fallback

- **GIVEN** no allow/ask/deny rule matches the tool execution
- **AND** mode is `AcceptEdits`
- **WHEN** agent attempts to execute `write_file` tool
- **THEN** tool executes immediately (mode allows)
- **WHEN** agent attempts to execute `run_command` tool
- **THEN** system emits permission request (mode requires ask for non-edit tools)

### Requirement: Backward Compatibility Layer

The system SHALL maintain 100% backward compatibility with legacy permission fields (edit, bash, skill, overrides) when new rule lists are empty.

#### Scenario: Empty rule lists trigger legacy behavior

- **GIVEN** AgentConfig with allow=[], ask=[], deny=[]
- **AND** edit field is `Ask`, bash field is `Allow`, skill field is `Deny`
- **WHEN** agent resolves permissions for any tool
- **THEN** legacy edit/bash/skill fields are used (exact match for existing behavior)
- **AND** rule-based resolution is not attempted
- **AND** mode field is ignored (default to `Default`)

#### Scenario: Non-empty rule lists take precedence

- **GIVEN** AgentConfig with allow=["Bash(git:*)"]
- **AND** legacy edit/bash/skill fields are set to `Allow`
- **WHEN** agent resolves permissions for Bash tool
- **THEN** rule-based resolution is used
- **AND** legacy edit/bash/skill fields are ignored
- **AND** deprecation warning is logged if legacy fields are non-Default

#### Scenario: Legacy overrides still work in legacy mode

- **GIVEN** AgentConfig with empty rule lists
- **AND** overrides map contains `{"custom_tool": "Allow"}`
- **WHEN** agent resolves permissions for `custom_tool`
- **THEN** override value is used (legacy behavior)
- **AND** result matches pre-refactor behavior exactly

### Requirement: Agent Configuration
The system SHALL support configuring Agents with strictly typed metadata including id, name, mode, permissions, and model settings.

#### Scenario: Load Valid Config with id
- **WHEN** a valid agent configuration JSON/TOML is loaded with both `id` and `name` fields
- **THEN** system correctly parses id, name, mode, and permission rules
- **AND** `id` is used as stable routing identifier for registry lookups
- **AND** `name` is used as display label for API responses

#### Scenario: Agent routing uses stable IDs
- **GIVEN** an Agent with `id="plan"` and `name="Plan Agent"`
- **WHEN** agent is registered in the registry
- **THEN** registry key is `"plan"` (not `"Plan Agent"`)
- **AND** API endpoints look up agent by `"plan"` (not by name)
- **AND** changing `name` to `"Planning Agent"` does not break existing clients using `"plan"`

### Requirement: Agent Permissions

The system SHALL enforce granular permissions for privileged operations through a hybrid system supporting both legacy categorization (edit/bash/skill) and modern rule-based lists (allow/ask/deny), with permission modes for different operational contexts.

#### Scenario: Ask requires explicit user approval and is resumable

- **GIVEN** tool execution is set to Ask
- **WHEN** agent attempts to execute a tool call with id `C1`
- **THEN** tool MUST NOT be executed automatically
- **AND** system MUST emit a permission-request signal with fields:
  - `operation`: `tool_execution`
  - `tool_name`: requested tool name
  - `call_id`: `C1`
  - `matched_rule`: The specific rule that triggered the Ask (if applicable)
  - `mode`: Current PermissionMode
- **AND** system MUST stop the current assistant turn until an explicit approval decision is received

#### Scenario: Multiple tool calls respect order and block on permission

- **GIVEN** a single assistant message includes tool calls `C1`, `C2`, and `C3` in that order
- **AND** `C1` is Ask-gated
- **AND** `C2` is Allow-gated
- **AND** `C3` is Ask-gated
- **WHEN** agent processes the assistant message tool calls
- **THEN** system MUST emit a permission request for `C1`
- **AND** system MUST NOT execute `C2` or `C3`
- **AND** system MUST NOT emit a permission request for `C3`
- **AND** system MUST stop the current assistant turn waiting for `C1`

#### Scenario: Resuming batch executes subsequent allowed tools

- **GIVEN** a blocked tool batch `[C1(Ask), C2(Allow), C3(Ask)]` stopped at `C1`
- **WHEN** user approves `C1`
- **THEN** system MUST execute `C1`
- **AND** system MUST execute `C2` immediately (as it is Allow-gated)
- **AND** system MUST emit a permission request for `C3`
- **AND** system MUST stop the current assistant turn waiting for `C3`

#### Scenario: Deny appends deterministic tool-result and continues to same assistant turn

- **GIVEN** a pending permission request for tool call id `C1`
- **WHEN** user denies execution for `C1`
- **THEN** system MUST NOT execute the tool call
- **AND** system MUST append a Tool message correlated to `C1` with exact content:
  - `Permission denied: user rejected tool execution.`
- **AND** system MUST perform the next model call to continue the same assistant turn

#### Scenario: Legacy edit/bash/skill categorization works when rules empty

- **GIVEN** an AgentConfig with legacy fields only (no allow/ask/deny rules)
- **AND** `edit` field is set to `Allow`
- **AND** `bash` field is set to `Ask`
- **AND** `skill` field is set to `Deny`
- **WHEN** agent attempts to execute `write_file` tool
- **THEN** tool is allowed without prompting
- **WHEN** agent attempts to execute `run_command` tool
- **THEN** system emits permission request (Ask behavior)
- **WHEN** agent attempts to execute `read_file` tool
- **THEN** tool is denied without prompting

### Requirement: Subagent Permission Mode Override

The system SHALL allow subagents to override the parent agent's permission mode for isolation and specialization.

#### Scenario: Subagent uses stricter permission mode

- **GIVEN** parent agent mode is `Default`
- **AND** subagent config specifies mode as `Plan`
- **WHEN** subagent attempts to execute `write_file` tool
- **THEN** tool is denied (Plan mode applies)
- **AND** parent agent's Default mode does not apply to subagent

#### Scenario: Subagent inherits parent rules when mode not overridden

- **GIVEN** parent agent has allow list containing `"Read(./src/**)"`
- **AND** subagent config does not specify mode
- **WHEN** subagent attempts to read `./src/agent.rs`
- **THEN** file is read immediately (inherited rule)
- **AND** subagent uses parent's mode (Default)

### Requirement: Dynamic System Prompt
The system SHALL generate system prompts dynamically at the start of a user chat turn using Jinja2 templates with semantic XML-style tags, including current environment context and custom project rules.

#### Scenario: Environment injection uses XML tags
- **GIVEN** a system prompt template with environment variables
- **WHEN** a system prompt is generated
- **THEN** it includes current Working Directory, Platform (OS), and Today's Date in an `<environment>` XML tag

The `<environment>` tag MUST contain these labels exactly:
- `OS:`
- `CWD:`
- `Date:`

#### Scenario: System prompt uses semantic XML structure
- **WHEN** a system prompt is generated
- **THEN** it MUST use following XML tags for semantic chunking:
  - `<instructions>`: Core instructions from AgentConfig
  - `<environment>`: Environment context (OS, CWD, Date, workspace_root)
  - `<project_rules>`: AGENTS.md content if present

#### Scenario: Custom rules injection from AGENTS.md
- **GIVEN** `AGENTS.md` exists in workspace root
- **WHEN** a system prompt is generated
- **THEN** its content is wrapped in a `<project_rules>` XML tag and appended to system prompt

#### Scenario: Project rules content is escaped
- **GIVEN** `AGENTS.md` contains XML-special characters (`<`, `>`, `&`)
- **WHEN** its content is embedded into `<project_rules>`
- **THEN** XML-special characters are escaped so they cannot create new tags

#### Scenario: Prompt is stable within a user turn
- **GIVEN** a single user input triggers multiple completion requests due to tool calls
- **WHEN** agent rebuilds completion requests for that user input
- **THEN** system prompt content (including all XML tags) remains unchanged across those requests

This stability requirement includes:
- The `<environment>` tag values, which are snapshotted at the start of user chat turn
- The `<project_rules>` tag content, which is snapshotted at the start of user chat turn
- All other XML tag content, which must not change during turn

#### Scenario: Template variable interpolation works
- **GIVEN** a system prompt template `You are {{instructions}}. OS: {{os}}`
- **AND** context variables `instructions = "a helpful assistant"`, `os = "linux"`
- **WHEN** the system prompt template is rendered
- **THEN** output is `You are a helpful assistant. OS: linux`

#### Scenario: Template conditionals work
- **GIVEN** a system prompt template with conditional section
- **WHEN** the system prompt template is rendered with appropriate context
- **THEN** conditional sections are included or excluded based on Jinja2 `{% if %}` syntax

#### Scenario: Template loops work
- **GIVEN** a system prompt template with a loop over a list variable
- **WHEN** the system prompt template is rendered
- **THEN** the loop iterates correctly and outputs multiple items

#### Scenario: Template error includes context
- **GIVEN** a system prompt template with undefined variable or syntax error
- **WHEN** the system prompt template is compiled or rendered
- **THEN** a clear error message is returned indicating:
  - The undefined variable name or syntax error description
  - The line number where the error occurred (if available)
  - A suggested fix or reference to documentation

#### Scenario: Template validation at load time
- **GIVEN** a custom system prompt template file is provided
- **WHEN** the agent starts
- **THEN** the template is validated for Jinja2 syntax before first use
- **AND** if validation fails, a clear error message is logged indicating the template source and error details

#### Scenario: Backward compatibility (no template specified)
- **GIVEN** an agent configuration with no custom template specified
- **WHEN** the system prompt is generated
- **THEN** the system prompt builder uses a default embedded template
- **AND** the default template renders with the same behavior as the current format! macro approach

#### Scenario: Migration path is seamless
- **GIVEN** existing agent configuration with only `instructions` field (no template)
- **WHEN** system prompt is generated without template specified
- **THEN** output matches current behavior exactly (same format, same content)
- **AND** no configuration changes are required
- **AND** existing agent behavior is preserved

#### Scenario: Template discovery order respects configuration
- **GIVEN** an agent configuration with `system_prompt_template` field set
- **AND** a custom template file exists at `.sisyphus/templates/system_prompt.jinja`
- **WHEN** the system prompt is generated
- **THEN** the inline `system_prompt_template` field is used (highest priority)
- **AND** the file-based template is ignored

### Requirement: Built-in Repository Search Tools
The system SHALL expose `glob` and `grep` as callable tools in the agent tool definitions.

#### Scenario: Tools are available to the model
- **GIVEN** an agent session is started
- **WHEN** the system creates the completion request tool definitions
- **THEN** the tool list includes `glob`
- **AND** the tool list includes `grep`

### Requirement: Repository Search Tool Schemas
The system SHALL expose stable tool schemas for `glob` and `grep` so models can call them reliably.

#### Scenario: Tool schemas include required arguments
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** `glob` schema requires `pattern`
- **AND** `grep` schema requires `pattern`

#### Scenario: Tool schemas support selective ignore overrides
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include an `include_ignored` argument

#### Scenario: Tool schemas include output cap arguments
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include a `max_results` argument

#### Scenario: Tool schemas include optional base path
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include an optional `path` argument

#### Scenario: Grep schema supports output modes
- **GIVEN** `grep` is registered
- **WHEN** the system exposes its tool schema
- **THEN** the schema includes an `output_mode` argument
- **AND** `output_mode` supports `files_with_matches`, `content`, and `count`

### Requirement: System Prompt Construction
The system prompt builder MUST NOT block the execution thread when reading environment context or external files.

#### Scenario: Reading custom rules
- **GIVEN** a large `AGENTS.md` file
- **WHEN** the agent builds the system prompt for a new user turn
- **THEN** it reads the file asynchronously without blocking the agent runtime

### Requirement: Environment Snapshotting
The agent MUST capture a snapshot of the environment (OS, workspace root, date, custom rules) asynchronously before generating the system prompt for each user turn.

#### Scenario: Capturing snapshot
- **GIVEN** a running agent
- **WHEN** a new user turn begins
- **THEN** it captures the environment state asynchronously before the first model call

#### Scenario: Snapshot reuse within a turn
- **GIVEN** a single user turn that triggers tool calls
- **WHEN** the agent makes multiple model calls to complete the turn
- **THEN** it reuses the same environment snapshot for that turn

### Requirement: Workspace Rule Resolution
The agent MUST resolve `AGENTS.md` from the workspace root used for tool sandboxing, not from an implicit process working directory.

#### Scenario: Deterministic rule discovery
- **GIVEN** a workspace root containing `AGENTS.md`
- **WHEN** the agent builds the system prompt
- **THEN** it includes the contents of that workspace-root `AGENTS.md`

### Requirement: Expose Agent Metadata via Server API
The system SHALL expose Agent configuration metadata, including model settings, via the server Agent discovery API.

#### Scenario: Agent Metadata Reflects Config
- **GIVEN** an Agent configured with a specific name, description, mode, permissions, and model settings
- **WHEN** the server returns that Agent via `GET /api/v1/agents` or `GET /api/v1/agents/:id`
- **THEN** the returned metadata MUST include the configured name, description, and model
- **AND** it MUST NOT expose internal-only fields that are not part of the Agent configuration contract.

### Requirement: Jinja2 Template Engine
The system SHALL use the minijinja crate for Jinja2-compatible template processing in SlashCommand templates.

#### Scenario: Jinja2 variable interpolation works
- **GIVEN** a template string `Hello {{name}}!`
- **AND** a context variable `name = "World"`
- **WHEN** the template is rendered
- **THEN** the output is `Hello World!`

#### Scenario: Jinja2 conditionals work
- **GIVEN** a template string `{% if show_message %}Message shown{% endif %}`
- **AND** a context variable `show_message = true`
- **WHEN** the template is rendered
- **THEN** the output includes `Message shown`
- **AND** when `show_message = false`, the output is empty

#### Scenario: Jinja2 loops work
- **GIVEN** a template string `{% for item in items %}{{item}} {% endfor %}`
- **AND** a context variable `items = ["a", "b", "c"]`
- **WHEN** the template is rendered
- **THEN** the output is `a b c `

#### Scenario: Jinja2 filters work
- **GIVEN** a template string `{{text|upper}}`
- **AND** a context variable `text = "hello"`
- **WHEN** the template is rendered
- **THEN** the output is `HELLO`

#### Scenario: Template error includes context
- **GIVEN** a template string with undefined variable `{{undefined_var}}`
- **WHEN** the template is rendered
- **THEN** a clear error is returned indicating:
  - The undefined variable name
  - The line number where the error occurred
  - The template source (if available)

### Requirement: Backward Compatibility for Template Syntax
The system MUST maintain backward compatibility with existing `{{args}}` syntax in custom SlashCommand templates.

#### Scenario: Existing `{{args}}` templates continue to work
- **GIVEN** a custom SlashCommand with template `Hello {{args}}`
- **AND** user executes `/command World`
- **WHEN** the template is rendered
- **THEN** the output is `Hello World`
- **AND** no migration or modification is required

#### Scenario: Mix of old and new syntax works
- **GIVEN** a custom SlashCommand with template `Hello {{args}}, {{args|upper}}`
- **AND** user executes `/command World`
- **WHEN** the template is rendered
- **THEN** both `{{args}}` and the Jinja2 filter work correctly
- **AND** the output is `Hello World, WORLD`

### Requirement: Built-in Agent Definitions
The system SHALL provide two built-in agents with stable identifiers and distinct tool sets: a read-only `plan` agent for analysis, and an execution-capable `build` agent for implementation.

#### Scenario: Default agent is Plan agent
- **GIVEN** a new session is created without specifying an `agent_id`
- **WHEN** session initializes
- **THEN** it defaults to `plan` agent
- **AND** `plan` agent is read-only by tool exposure and permission policy

#### Scenario: Plan agent is read-only
- **GIVEN** `plan` agent is configured
- **WHEN** tools are registered on the plan agent
- **THEN** only read-only tools are registered (read_file, glob, grep)
- **AND** write_file, replace_in_file, and execute_command tools are NOT registered
- **AND** permission policy denies edit and command execution operations

#### Scenario: Agent discovery returns both agents
- **GIVEN** both `plan` and `build` agents are registered
- **WHEN** a client requests `GET /api/v1/agents`
- **THEN** response includes two agents with distinct IDs
- **AND** each agent includes: `id`, `name`, `description`, and `model`

#### Scenario: Agent selection by ID works
- **GIVEN** both agents are registered in the registry
- **WHEN** a client creates a session with `agent_id: "plan"`
- **THEN** session uses Plan agent
- **AND** subsequent tool calls are routed to Plan agent
- **AND** Plan agent cannot call write_file or execute_command (permission deny)

#### Scenario: Agent switching works mid-session
- **GIVEN** an active session using Build agent
- **WHEN** a client sends `PUT /api/v1/sessions/:id/agent` with `agent_id: "plan"`
- **THEN** session switches to Plan agent
- **AND** subsequent chat requests are routed to Plan agent
- **AND** Plan agent's tool restrictions apply immediately

### Requirement: Agent Identity Separation
The system SHALL separate agent routing identity (`id`) from display identity (`name`) to enable stable client integration.

#### Scenario: Display name changes do not break routing
- **GIVEN** an agent with `id="plan"` and `name="Plan Agent"`
- **WHEN** agent's display name is changed to `"Planning Agent"` in a future update
- **THEN** routing ID `"plan"` remains unchanged
- **AND** existing clients using `agent_id: "plan"` continue to work
- **AND** new API responses show updated display name

### Requirement: Tool Registration Validation
The system SHALL provide a helper function to register read-only tools with validation to prevent accidental registration of state-changing tools on Plan agent.

#### Scenario: Read-only tool registration validates allowed tools
- **GIVEN** Plan agent's `register_read_only_tools()` helper is called
- **WHEN** attempting to register `write_file` tool
- **THEN** registration fails with an error message indicating tool is not allowed
- **AND** tool is NOT added to agent
- **AND** error message includes list of allowed tool names

#### Scenario: Read-only tool registration accepts allowed tools
- **GIVEN** Plan agent's `register_read_only_tools()` helper is called
- **WHEN** attempting to register `read_file` tool
- **THEN** registration succeeds
- **AND** tool is available for agent to use

### Requirement: Shared LLM Provider
The system SHALL allow initializing multiple built-in agents derived from the same runtime LLM configuration.

#### Scenario: Multiple agents share one provider
- **GIVEN** an LLM provider is configured
- **WHEN** both Plan and Build agents are initialized
- **THEN** both agents use the same model configuration

### Requirement: Agent Unit Test Organization
The system SHALL organize agent tests into `#[cfg(test)]` modules for fast execution and integration tests in `tests/` directory for end-to-end flows.

#### Scenario: Unit tests use mocks
- **GIVEN** agent unit tests in `src/agent.rs`
- **WHEN** tests are run
- **THEN** all tests use mock LLM providers
- **AND** all tests use mock tools
- **AND** no external dependencies are required

#### Scenario: Integration tests use real components
- **GIVEN** agent integration tests in `tests/`
- **WHEN** tests are run
- **THEN** tests verify agent loop with real session
- **AND** tests use scripted/mock providers
- **AND** tests verify permission flows

### Requirement: Mock Framework Adoption
The system SHALL use `mockall` framework for creating mock LLM providers and tools in agent tests.

#### Scenario: Mock LLM provider
- **GIVEN** an agent test requiring mock provider
- **WHEN** test sets up `MockLLMProvider::new()`
- **THEN** mockall creates a valid mock object
- **AND** expectations can be set on the mock
- **AND** mock returns scripted responses

#### Scenario: Mock tool expectations
- **GIVEN** an agent test requiring mock tool
- **WHEN** test configures mock tool expectations
- **THEN** mockall verifies expected calls
- **AND** mockall verifies call arguments
- **AND** mockall returns specified results

### Requirement: Snapshot Testing for Prompts
The system SHALL use `insta` for snapshot testing of generated system prompts to detect unintended changes.

#### Scenario: System prompt snapshot
- **GIVEN** an agent configuration with instructions
- **WHEN** system prompt is generated
- **THEN** snapshot matches expected output
- **AND** XML tags are correctly formatted
- **AND** environment variables are included

#### Scenario: Template prompt snapshot
- **GIVEN** an agent configuration with custom template
- **WHEN** system prompt is generated from template
- **THEN** snapshot matches expected output
- **AND** template variables are interpolated correctly
- **AND** conditional sections work as expected

