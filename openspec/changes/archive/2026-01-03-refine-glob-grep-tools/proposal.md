# Change: Refine glob and grep tools

## Why
The initial `glob` and `grep` tools improve safety and repository exploration, but there are a few behavior and spec mismatches (notably `grep` column semantics) and some edge cases (binary/non-UTF-8 files and invalid override patterns) that should be addressed to make results predictable, deterministic, and secure.

## What Changes
- Define `grep` content result `column` as a 1-based character index (Unicode scalar values) within the returned `text` line.
- Define deterministic handling for binary and non-UTF-8 files (skip with no results).
- Define deterministic line truncation rules for `grep` content output so returned `text` always contains the match.
- Require invalid `include_ignored` / `exclude` patterns to fail fast with a clear error.
- Require invalid `grep` regular expressions to fail fast with a clear error.
- Refactor shared sandbox/denylist/path validation logic between `glob` and `grep` to reduce duplication and drift.

## Impact
- Affected specs: `tooling` (glob/grep behavior and output semantics).
- Affected code (expected): `crates/tools/src/glob.rs`, `crates/tools/src/grep.rs`, and shared helpers in `crates/tools`.

## Dependencies
- This change assumes `add-glob-grep-tools` is applied/archived so the `tooling` capability exists as current truth.

## Non-Goals
- Full ripgrep CLI parity.
- New search features such as multiline mode, context lines, or streaming output.
