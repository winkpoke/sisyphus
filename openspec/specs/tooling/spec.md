# Tooling Specification

## Purpose
Defines the standard set of tools available to agents, including file discovery and search capabilities.
## Requirements
### Requirement: Glob Tool
The system SHALL treat `glob` as a Parallel tool.

#### Scenario: Glob executes under parallel mode
- **GIVEN** `glob` is registered
- **WHEN** the agent schedules multiple `glob` tool calls
- **THEN** the runtime MAY execute those calls concurrently

### Requirement: Grep Tool
The system SHALL treat `grep` as a Parallel tool.

#### Scenario: Grep executes under parallel mode
- **GIVEN** `grep` is registered
- **WHEN** the agent schedules multiple `grep` tool calls
- **THEN** the runtime MAY execute those calls concurrently

### Requirement: Replace In File Tool
The system SHALL provide a `replace_in_file` tool for replacing exact text patterns in files.

The `replace_in_file` tool MUST accept a JSON object argument with the following fields:
- `path` (string, required): Relative path to file.
- `old_string` (string, required): Exact text to search for and replace (case-sensitive).
- `new_string` (string, required): New text to replace `old_string` with.
- `replace_all` (boolean, optional, default `false`): If `true`, replace all occurrences; if `false`, replace only the first occurrence.

The `replace_in_file` tool MUST:
- Validate that file exists before attempting modification.
- Perform exact string matching (case-sensitive, not regex).
- Return an error if `old_string` is not found in the file.
- Read the current file content, perform replacement, and write back.
- Use sandboxed path resolution to prevent path traversal.

#### Scenario: Single replacement succeeds
- **GIVEN** a workspace with a file containing text `Hello World`
- **WHEN** `replace_in_file` is called with `path="test.txt"`, `old_string="World"`, `new_string="Universe"`, `replace_all=false`
- **THEN** the tool returns success
- **AND** the file content is updated to `Hello Universe`

#### Scenario: All occurrences replaced
- **GIVEN** a workspace with a file containing text `foo foo bar foo`
- **WHEN** `replace_in_file` is called with `path="test.txt"`, `old_string="foo"`, `new_string="baz"`, `replace_all=true`
- **THEN** the tool returns success
- **AND** the file content is updated to `baz baz bar baz`

#### Scenario: Pattern not found returns error
- **GIVEN** a workspace with a file containing text `Hello World`
- **WHEN** `replace_in_file` is called with `path="test.txt"`, `old_string="Goodbye"`
- **THEN** the tool returns an error indicating that pattern was not found
- **AND** the file is not modified

#### Scenario: File not found returns error
- **GIVEN** a workspace without `nonexistent.txt`
- **WHEN** `replace_in_file` is called with `path="nonexistent.txt"`
- **THEN** the tool returns an error indicating that file was not found

### Requirement: Permission Configuration
The system MUST support configuring permissions per tool via configuration, without requiring code changes for standard tools, and it MUST preserve safe defaults for tools that are not explicitly configured.

#### Scenario: Overriding tool permission
- **GIVEN** an agent configuration
- **WHEN** I set `permissions.overrides["my_tool"] = "Deny"`
- **THEN** `my_tool` is denied execution
- **AND** other tools follow their default permission behavior

#### Scenario: Unspecified tools remain safe
- **GIVEN** an agent configuration with no override for `new_tool`
- **WHEN** `new_tool` is requested for execution
- **THEN** `new_tool` is evaluated under a safe default policy (Ask/Deny)

### Requirement: File System Tool Security Testing
The system SHALL provide comprehensive security tests for file system operations in `tools/src/fs.rs` to prevent path traversal, unauthorized access, and sandbox escapes.

#### Scenario: Sandboxed file operations
- **GIVEN** a test using `tempfile::tempdir()`
- **WHEN** file operations are performed
- **THEN** all operations are confined to temp directory
- **AND** no files are created outside sandbox
- **AND** temp directory is cleaned up after test

#### Scenario: Path traversal prevention
- **GIVEN** a file system tool
- **WHEN** path includes `../` for traversal
- **THEN** operation is rejected with error
- **AND** no files outside workspace are accessed
- **AND** error message indicates security violation

#### Scenario: Absolute path rejection
- **GIVEN** a file system tool configured for workspace-relative paths
- **WHEN** an absolute path is provided
- **THEN** operation is rejected with error
- **AND** error message indicates path must be relative
- **AND** no files are accessed outside workspace

#### Scenario: Symlink escape prevention
- **GIVEN** a sandboxed directory containing symlinks
- **AND** symlink points outside sandbox
- **WHEN** file operation attempts to follow symlink
- **THEN** operation is rejected or confined
- **AND** sandbox boundary is not violated

#### Scenario: Permission error handling
- **GIVEN** a file system tool
- **AND** file exists with read-only permissions
- **WHEN** write operation is attempted
- **THEN** operation returns error
- **AND** error indicates permission denied
- **AND** file is not modified

#### Scenario: Non-existent file handling
- **GIVEN** a file system tool
- **WHEN** read operation attempts non-existent file
- **THEN** operation returns error
- **AND** error indicates file not found
- **AND** no partial reads occur

#### Scenario: Large file handling
- **GIVEN** a file system tool
- **AND** large file is requested (>100MB)
- **WHEN** read operation is attempted
- **THEN** operation handles gracefully
- **AND** appropriate error or truncation occurs
- **AND** no resource exhaustion

### Requirement: Shell Tool Security Testing
The system SHALL provide comprehensive security tests for shell command execution in `tools/src/cmd.rs` to prevent command injection, privilege escalation, and dangerous operations.

#### Scenario: Safe command execution
- **GIVEN** a shell tool with allowlist of safe commands
- **WHEN** safe command is executed (echo, ls, pwd)
- **THEN** command executes successfully
- **AND** output is captured
- **AND** exit code is returned

#### Scenario: Dangerous command blocking
- **GIVEN** a shell tool with blocklist of dangerous commands
- **WHEN** dangerous command is attempted (rm, sudo, dd)
- **THEN** operation is rejected
- **AND** error indicates command not allowed
- **AND** command is not executed

#### Scenario: Shell injection prevention (semicolons)
- **GIVEN** a shell tool
- **WHEN** command argument contains semicolon separator (`;`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid characters

#### Scenario: Shell injection prevention (pipe)
- **GIVEN** a shell tool
- **WHEN** command argument contains pipe character (`|`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid characters

#### Scenario: Shell injection prevention (ampersand)
- **GIVEN** a shell tool
- **WHEN** command argument contains background operator (`&`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid characters

#### Scenario: Shell injection prevention (backticks)
- **GIVEN** a shell tool
- **WHEN** command argument contains backticks (`` ` `` ``)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid characters

#### Scenario: Shell injection prevention (command substitution)
- **GIVEN** a shell tool
- **WHEN** command argument contains command substitution (`$(rm -rf /)`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid syntax

#### Scenario: Shell injection prevention (environment variable)
- **GIVEN** a shell tool
- **WHEN** command argument attempts to set environment variable (`HOME=/tmp`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid input

#### Scenario: Shell injection prevention (heredoc)
- **GIVEN** a shell tool
- **WHEN** command argument contains heredoc syntax (`<<EOF`)
- **THEN** injection attempt is detected
- **AND** command is rejected or sanitized
- **AND** error indicates invalid syntax

#### Scenario: Command argument escaping
- **GIVEN** a shell tool
- **WHEN** command argument contains special characters (spaces, quotes)
- **THEN** arguments are properly escaped
- **AND** command executes as intended
- **AND** no shell interpretation occurs

#### Scenario: Timeout handling
- **GIVEN** a shell tool with timeout configured
- **WHEN** command execution exceeds timeout
- **THEN** process is terminated
- **AND** timeout error is returned
- **AND** no zombie processes remain

#### Scenario: Stdout/stderr capture
- **GIVEN** a shell tool
- **WHEN** command produces output to both stdout and stderr
- **THEN** both streams are captured
- **AND** output is properly separated
- **AND** command exit code is determined correctly

#### Scenario: Environment variable injection prevention
- **GIVEN** a shell tool
- **WHEN** command attempts to set environment variable
- **THEN** injection attempt is detected
- **AND** variable setting is rejected or sanitized
- **AND** error indicates invalid input

### Requirement: Tool Execution Timeout
The system SHALL provide timeout mechanism for all tool executions to prevent hanging operations.

#### Scenario: Timeout enforcement
- **GIVEN** a tool with timeout configured
- **WHEN** tool execution exceeds timeout duration
- **THEN** execution is terminated
- **AND** timeout error is returned
- **AND** partial results are not processed

#### Scenario: Timeout cancellation
- **GIVEN** a tool execution in progress
- **WHEN** timeout is triggered
- **THEN** execution is cancelled immediately
- **AND** resources are cleaned up
- **AND** no partial state is persisted

#### Scenario: Test cleanup after timeout
- **GIVEN** a shell tool test with timeout
- **WHEN** command exceeds timeout and is terminated
- **THEN** zombie processes are cleaned up
- **AND** no file descriptors remain open
- **AND** cleanup is verified in test teardown

#### Scenario: Temp directory cleanup after shell test
- **GIVEN** a shell tool test using tempfile
- **WHEN** test completes (success or failure)
- **THEN** all temp files and directories are cleaned up
- **AND** no test artifacts remain on disk
- **AND** cleanup is verified in test teardown

### Requirement: Tool execution modes
The system SHALL assign each registered tool an execution mode: Parallel or Sequential.

#### Scenario: Default execution mode is Sequential
- **GIVEN** a tool is registered without an explicit execution mode
- **WHEN** the tool is executed
- **THEN** it is executed in Sequential mode

### Requirement: Selective parallel execution
The system SHALL allow concurrent execution of Parallel tools while ensuring Sequential tools execute exclusively.

#### Scenario: Parallel tools can run concurrently
- **GIVEN** two Parallel tool calls are scheduled
- **WHEN** the system executes them
- **THEN** they MAY execute at the same time

#### Scenario: Sequential tool blocks all other tools
- **GIVEN** a Sequential tool call is scheduled
- **WHEN** the system begins executing it
- **THEN** no other tool call executes until it completes

