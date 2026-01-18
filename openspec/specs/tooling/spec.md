# Tooling Specification

## Purpose
Defines the standard set of tools available to agents, including file discovery and search capabilities.
## Requirements
### Requirement: Glob Tool
The system SHALL provide a `glob` tool for discovering files under the workspace root.

The `glob` tool MUST accept a JSON object argument with the following fields:
- `pattern` (string, required): Glob pattern evaluated against paths relative to `path`.
- `path` (string, optional, default `"."`): Workspace-relative base directory to search under.
- `include_ignored` (array of strings, optional, default `[]`): Glob patterns that selectively re-include otherwise ignored paths.
- `exclude` (array of strings, optional, default `[]`): Additional glob patterns to exclude.
- `max_results` (integer, optional, default 1000): Maximum number of file paths returned.

The `glob` tool MUST:
- Reject invalid glob syntax in `pattern`, `include_ignored`, or `exclude` with an error.
- Return only regular files (not directories).
- Treat `path` as workspace-relative and reject absolute paths and traversal.
- Normalize returned paths to workspace-relative strings using `/` separators.
- Produce deterministic results: eligible paths MUST be ordered by ascending normalized path string.
- If `max_results` is reached, return the first `max_results` results in that deterministic order.

The `glob` tool MUST return a UTF-8 string containing a JSON object with the following fields:
- `paths` (array of strings): Workspace-relative, normalized file paths.
- `truncated` (boolean): `true` if results were capped by `max_results`.

#### Scenario: Find Rust files under crates
- **GIVEN** a workspace containing `crates/tools/src/cmd.rs`
- **WHEN** `glob` is called with `path="."` and `pattern="crates/**/*.rs"`
- **THEN** the returned `paths` includes `crates/tools/src/cmd.rs`

#### Scenario: Glob excludes directories
- **GIVEN** a workspace containing a directory `crates/tools/src/`
- **WHEN** `glob` is called with `path="."` and `pattern="crates/tools/src"`
- **THEN** the returned `paths` does not include `crates/tools/src`

#### Scenario: Glob output ordering is deterministic
- **GIVEN** a workspace with multiple matching files
- **WHEN** `glob` is called
- **THEN** the returned `paths` is sorted by ascending normalized path string

#### Scenario: Glob truncation is deterministic
- **GIVEN** a workspace where more than 2 paths match a pattern
- **WHEN** `glob` is called with `max_results=2`
- **THEN** it returns exactly 2 paths
- **AND** those 2 paths are the first 2 entries in the deterministic sort order
- **AND** it sets `truncated=true`

#### Scenario: Invalid include_ignored pattern
- **GIVEN** `glob` is registered
- **WHEN** `glob` is called with an invalid pattern in `include_ignored`
- **THEN** the tool returns an error indicating the pattern is invalid

#### Scenario: Invalid exclude pattern
- **GIVEN** `glob` is registered
- **WHEN** `glob` is called with an invalid pattern in `exclude`
- **THEN** the tool returns an error indicating the pattern is invalid

### Requirement: Grep Tool
The system SHALL provide a `grep` tool that searches file contents using a regular expression and returns bounded results.

The `grep` tool MUST accept a JSON object argument with the following fields:
- `pattern` (string, required): Regular expression to search for.
- `path` (string, optional, default `"."`): Workspace-relative base directory to search under.
- `output_mode` (string, optional, default `"files_with_matches"`): One of `files_with_matches`, `content`, `count`.
- `include_ignored` (array of strings, optional, default `[]`): Glob patterns that selectively re-include otherwise ignored paths.
- `exclude` (array of strings, optional, default `[]`): Additional glob patterns to exclude.
- `max_results` (integer, optional, default 100): Maximum number of result entries returned.
- `max_file_bytes` (integer, optional, default 1000000): Maximum number of bytes read per file.
- `max_line_length` (integer, optional, default 2000): Maximum line length included in `content` results.

The `grep` tool MUST:
- Interpret `pattern` using Rust `regex` crate syntax.
- Reject invalid regular expressions in `pattern` with an error.
- Reject invalid glob syntax in `include_ignored` or `exclude` with an error.
- Treat `path` as workspace-relative and reject absolute paths and traversal.
- Normalize returned paths to workspace-relative strings using `/` separators.
- Enforce the configured caps and terminate early when `max_results` is reached.
- Skip binary files deterministically.

Binary files MUST be defined as files where any of the first `max_file_bytes` bytes either:
- Contain at least one NUL byte (`0x00`), or
- Are not valid UTF-8.

The `grep` tool MUST return a UTF-8 string containing a JSON object with the following fields:
- `output_mode` (string): Echo of the selected output mode.
- `results` (array): Mode-dependent result entries.
- `truncated` (boolean): `true` if results were capped by `max_results`.

For `output_mode="files_with_matches"`, `results` MUST be an array of workspace-relative path strings.

For `output_mode="count"`, `results` MUST be an array of objects with:
- `path` (string): Workspace-relative, normalized path.
- `count` (integer): Number of match occurrences in that file.

For `output_mode="content"`, `results` MUST be an array of objects with:
- `path` (string): Workspace-relative, normalized path.
- `line` (integer): 1-based line number of the match.
- `column` (integer): 1-based character index (Unicode scalar values) of the match start within `text`.
- `text` (string): The matching line, truncated to `max_line_length`.

For `output_mode="content"`:
- Each match occurrence MUST produce a separate result entry.
- `text` MUST be truncated from the start of the line to at most `max_line_length` characters.
- Any match whose start column would be greater than `max_line_length` MUST NOT be returned.

For all output modes, results MUST be ordered deterministically by ascending `path` and then by mode-specific stable keys (for `content`: `line`, then `column`).

#### Scenario: Search for a symbol in the repository
- **GIVEN** a workspace containing at least one file with the text `register_tool`
- **WHEN** `grep` is called with `path="."`, `pattern="register_tool"`, and `output_mode="files_with_matches"`
- **THEN** the returned `results` includes at least one matching file path

#### Scenario: Grep content mode returns line metadata
- **GIVEN** a workspace containing a file where a line includes the text `register_tool`
- **WHEN** `grep` is called with `output_mode="content"` and `pattern="register_tool"`
- **THEN** every entry in `results` includes `path`, `line`, `column`, and `text`

#### Scenario: Non-ASCII column semantics
- **GIVEN** a workspace containing a UTF-8 text file with non-ASCII characters
- **WHEN** `grep` is called with `output_mode="content"`
- **THEN** each `column` is the 1-based character index within the returned `text`

#### Scenario: Invalid regex fails fast
- **GIVEN** `grep` is registered
- **WHEN** `grep` is called with an invalid regular expression in `pattern`
- **THEN** the tool returns an error indicating the regex is invalid

#### Scenario: Binary files are skipped
- **GIVEN** a workspace containing a file with NUL bytes
- **WHEN** `grep` is called with any `output_mode`
- **THEN** the tool does not return results from that file

#### Scenario: Invalid UTF-8 files are skipped
- **GIVEN** a workspace containing a file where the scanned content is not valid UTF-8
- **WHEN** `grep` is called with any `output_mode`
- **THEN** the tool does not return results from that file

#### Scenario: Content truncation contains the match
- **GIVEN** a workspace containing a UTF-8 text file with a line longer than `max_line_length`
- **WHEN** `grep` is called with `output_mode="content"` and a match that starts after `max_line_length`
- **THEN** the tool does not return a `content` entry for that match

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

