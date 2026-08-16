## MODIFIED Requirements

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

## ADDED Requirements

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
