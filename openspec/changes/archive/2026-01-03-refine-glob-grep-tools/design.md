## Context
`glob` and `grep` are internal tools exposed to the agent for repository exploration. Their outputs must be deterministic, bounded, and safe-by-default.

## Goals / Non-Goals
- Goals:
  - Eliminate ambiguity in `grep` content result column semantics.
  - Define consistent behavior for binary and invalid UTF-8 content.
  - Avoid silent failures on invalid override patterns.
  - Reduce duplicated sandbox/denylist validation logic.
- Non-Goals:
  - Implement advanced ripgrep features.
  - Change the high-level tool schemas.

## Decisions
- Decision: Define `grep` regular expression syntax as Rust `regex` crate syntax.
  - Why: This matches the current implementation and makes the regex dialect deterministic and documentable.
- Decision: Invalid `grep` regular expressions fail tool execution with a clear error.
  - Why: Silent fallback or partial matching is unpredictable and hard to debug.
- Decision: Define `column` as a 1-based character index (Unicode scalar values) within `text`.
  - Why: The `text` field is valid UTF-8 and is serialized as a JSON string; a character index is stable for consumers.
- Decision: `column` counts Unicode scalar values, not grapheme clusters or display cells.
  - Why: Grapheme and display-width semantics require additional libraries and are not stable across renderers.
- Decision: Treat files containing NUL bytes as binary and skip them.
  - Why: This prevents partial/invalid decoding and avoids unpredictable line parsing behavior.
- Decision: Treat files with invalid UTF-8 (within the scanned byte limit) as binary and skip them.
  - Why: JSON tool outputs are UTF-8; lossy decoding and partial-line skipping make results inconsistent.
- Decision: In `grep` content mode, ensure the returned `text` contains the match.
  - Why: Returning a truncated line that does not contain the match makes `column` unusable.
- Decision: Invalid `include_ignored` / `exclude` patterns cause tool execution to fail with a clear error.
  - Why: Silent ignore (or printing to stdout) causes hard-to-debug behavior and breaks determinism.
- Decision: Define deterministic traversal and truncation order.
  - Why: Early termination at `max_results` is only deterministic if discovery and result ordering are fully specified.

## Risks / Trade-offs
- Character-index columns differ from byte offsets.
  - Mitigation: Specify semantics precisely and test with non-ASCII input.
- Binary detection can misclassify some text files.
  - Mitigation: Use a conservative heuristic (NUL byte presence) and document it.
 - Treating invalid UTF-8 as binary may skip otherwise searchable content.
   - Mitigation: Keep behavior deterministic; users can use alternative tooling for arbitrary binary search if needed.
 - Dropping matches beyond `max_line_length` reduces recall on extremely long lines.
   - Mitigation: Prefer determinism and bounded output; users can raise `max_line_length` when needed.

## Migration Plan
- Apply this change after `add-glob-grep-tools` is archived into current specs.
- Implement behavior changes with unit tests to lock semantics.
