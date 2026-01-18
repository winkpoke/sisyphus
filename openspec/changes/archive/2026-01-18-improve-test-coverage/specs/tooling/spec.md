# tooling Specification (Delta)

## ADDED Requirements

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
