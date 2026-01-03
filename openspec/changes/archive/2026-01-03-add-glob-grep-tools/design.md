## Context
Sisyphus exposes tools to the LLM through an OpenAI-compatible function tool interface. Today, repository search is only possible by reading individual files or executing arbitrary shell commands. This proposal adds two dedicated search/navigation tools that are safe-by-default and optimized for agent workflows.

## Goals / Non-Goals
- Goals:
  - Provide `glob` and `grep` tools with clearly specified behavior and stable schemas.
  - Respect repository ignore rules by default and allow additional exclude patterns.
  - Allow selective overrides to include specific ignored paths without disabling ignore rules globally.
  - Ensure deterministic, bounded outputs suitable for LLM consumption.
  - Enforce sandbox boundaries and denylisted paths so tools cannot escape the workspace or read sensitive locations.
- Non-Goals:
  - Implement full `rg` CLI compatibility.
  - Add semantic or indexed search.
  - Add UI changes or streaming search output.

## Decisions
- Decision: Implement `glob` and `grep` as first-class internal tools (not shelling out).
  - Why: Avoid external binary dependency, improve portability, and enforce safety limits.
- Decision: Use gitignore-aware walking with stable ordering.
  - Why: Matches agent expectations and prevents scanning irrelevant vendor output by default.
- Decision: Support selective re-inclusion of ignored paths.
  - Why: Users often want to search a small ignored subtree (e.g., vendored code) without enabling ignored content globally.
- Decision: Always apply output caps and size limits.
  - Why: Prevent tool outputs from overwhelming the context window.
- Decision: Always apply a sensitive-path denylist that cannot be bypassed.
  - Why: `include_ignored` is a usability feature and must not become a secret-exfiltration mechanism.

## Alternatives Considered
- Shell out to `rg`/`fd`/`find`.
  - Pros: Minimal code.
  - Cons: Requires binaries, harder to sandbox/permission, harder to enforce strict output bounds.
- Vendor/bundle ripgrep binary.
  - Pros: Behavior parity.
  - Cons: Packaging complexity, platform-specific maintenance.

## Risks / Trade-offs
- Implementing search internally may miss edge-case parity with ripgrep.
  - Mitigation: Explicitly scope to a small, agent-relevant subset and add targeted tests.
- Large repos may still produce large match sets.
  - Mitigation: Enforce `max_results`, maximum bytes per file, and maximum line length in content output.
- Denylisting can hide legitimate files.
  - Mitigation: Keep the mandatory denylist minimal, configurable for additional patterns, and return a clear error when a request targets denylisted paths.

## Migration Plan
- Add tools and register them alongside existing file tools.
- Update prompts/config to prefer `glob`/`grep` for search workflows over shell execution.
- Keep `execute_command` available but reduce its necessity for navigation.

## Open Questions
- Should repository search tools have separate permission gating, or rely on denylist + sandboxing?
