## ADDED Requirements

### Requirement: Glob Tool
The system SHALL provide a `glob` tool for discovering files under the workspace root.

The `glob` tool MUST accept a JSON object argument with the following fields:
- `pattern` (string, required): Glob pattern evaluated against paths relative to `path`.
- `path` (string, optional, default `"."`): Workspace-relative base directory to search under.
- `include_ignored` (array of strings, optional, default `[]`): Glob patterns that selectively re-include otherwise ignored paths.
- `exclude` (array of strings, optional, default `[]`): Additional glob patterns to exclude.
- `max_results` (integer, optional, default 1000): Maximum number of file paths returned.

The `glob` tool MUST:
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
- Treat `path` as workspace-relative and reject absolute paths and traversal.
- Normalize returned paths to workspace-relative strings using `/` separators.
- Enforce the configured caps and terminate early when `max_results` is reached.

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
- `column` (integer): 1-based UTF-8 column number of the match start within the line.
- `text` (string): The matching line, truncated to `max_line_length`.

For all output modes, results MUST be ordered deterministically by ascending `path` and then by mode-specific stable keys (for `content`: `line`, then `column`).

#### Scenario: Search for a symbol in the repository
- **GIVEN** a workspace containing at least one file with the text `register_tool`
- **WHEN** `grep` is called with `path="."`, `pattern="register_tool"`, and `output_mode="files_with_matches"`
- **THEN** the returned `results` includes at least one matching file path

#### Scenario: Grep content mode returns line metadata
- **GIVEN** a workspace containing a file where a line includes the text `register_tool`
- **WHEN** `grep` is called with `output_mode="content"` and `pattern="register_tool"`
- **THEN** every entry in `results` includes `path`, `line`, `column`, and `text`

#### Scenario: Grep output ordering is deterministic
- **GIVEN** a workspace where multiple files match a pattern
- **WHEN** `grep` is called with `output_mode="files_with_matches"`
- **THEN** the returned `results` is sorted by ascending normalized path string

#### Scenario: Cap grep output
- **GIVEN** a workspace with many matches for a common pattern
- **WHEN** `grep` is called with `max_results=50`
- **THEN** it returns at most 50 result entries
- **AND** it sets `truncated=true`

### Requirement: Ignore Rules
The `glob` and `grep` tools MUST respect `.gitignore`-style ignore rules by default.

#### Scenario: Ignore node_modules
- **GIVEN** a repository with `node_modules/` ignored by `.gitignore`
- **WHEN** `grep` is called with `path="."` and a pattern that would match inside `node_modules/`
- **THEN** the tool MUST NOT return matches from `node_modules/`

### Requirement: Selective Ignore Overrides
When ignore rules are enabled, the `glob` and `grep` tools SHALL support selectively including specific ignored paths when explicitly requested.

The `include_ignored` patterns MUST be evaluated against paths relative to `path`.

#### Scenario: Include selected ignored paths
- **GIVEN** a repository where `.gitignore` ignores `A/`, `B/`, and `C/`
- **WHEN** `grep` is called with `path="."` and `include_ignored=["A/**","B/**"]`
- **THEN** the tool MUST be able to return matches from `A/` and `B/`
- **AND** the tool MUST NOT return matches from `C/`

#### Scenario: Include ignored does not disable ignore rules
- **GIVEN** a repository where `.gitignore` ignores `A/` and `B/`
- **WHEN** `grep` is called with `include_ignored=["A/**"]`
- **THEN** matches from `A/` are eligible to be returned
- **AND** matches from `B/` are not eligible to be returned

### Requirement: Sandbox Boundary
The `glob` and `grep` tools MUST only operate within the configured workspace root.

The tools MUST reject any `path` that is absolute or that attempts traversal.
Absolute paths include (but are not limited to) Windows drive paths (for example `C:\\...`) and UNC paths (for example `\\\\server\\share\\...`).

The tools MUST prevent escaping the workspace via symlinks by ensuring that any resolved filesystem path accessed during execution remains within the workspace root.

#### Scenario: Reject traversal
- **GIVEN** a workspace root at `.`
- **WHEN** `glob` is called with `path="../"`
- **THEN** the tool returns an error indicating traversal is not allowed

#### Scenario: Reject absolute paths
- **GIVEN** a workspace root at `.`
- **WHEN** `grep` is called with `path="C:\\"`
- **THEN** the tool returns an error indicating absolute paths are not allowed

#### Scenario: Prevent symlink escape
- **GIVEN** a workspace containing a symlink that resolves outside the workspace root
- **WHEN** `grep` is called with `path` targeting the symlinked subtree
- **THEN** the tool MUST NOT read files outside the workspace root

### Requirement: Sensitive Path Denylist
The `glob` and `grep` tools MUST deny access to sensitive paths.

If a request's effective `path` targets a denylisted subtree, the tool MUST return an error.
If a denylisted subtree exists beneath an otherwise allowed `path`, the tool MUST NOT read from that subtree.
The denylist MUST NOT be bypassable via `include_ignored`.

#### Scenario: Denylisted content cannot be searched
- **GIVEN** a repository containing a sensitive file under a denylisted path
- **WHEN** `grep` is called with an `include_ignored` pattern that targets the sensitive path
- **THEN** the tool returns an error indicating the target is denylisted
