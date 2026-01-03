# Change: Add glob and grep (ripgrep-style) tools

## Why
The agent currently relies on `execute_command` for repository search workflows, which is both unsafe and inefficient for common code navigation tasks. First-class `glob` and `grep` tools provide fast, predictable repository exploration with clear safety boundaries.

## What Changes
- Add a `glob` tool for fast file discovery using glob patterns.
- Add a `grep` tool for fast content search with regex support and bounded output.
- Respect repository ignore rules by default while allowing explicit, narrow re-inclusion.
- Allow callers to apply additional excludes without disabling ignore rules.
- Standardize tool schemas and deterministic ordering so agent behavior is reproducible.
- Enforce sandbox boundaries, denylisted paths, and output caps (results and size limits) to reduce data exfiltration risk.

## Impact
- Affected specs: `agent-core` (tool exposure and tool schemas), new `tooling` capability (search tools behavior).
- Affected code (expected): `crates/tools`, `crates/core` (tool definitions exposure), `crates/cli` (tool registration).

## Non-Goals
- Full ripgrep CLI parity (all flags and output formats).
- Streaming/interactive search UI changes.
- Indexing or semantic search.
