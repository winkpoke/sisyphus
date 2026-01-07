# Change: Upgrade SlashCommand templating and structure system prompts

## Why

The current custom SlashCommand expansion is limited to a single `{{args}}` string replacement, which makes templates hard to reuse and extend. Separately, system prompts are a flat concatenation, which makes prompt sections harder to visually scan.

This change keeps scope tight and aligned with current Sisyphus implementation.

## What Changes

- **Use Jinja2 templating for SlashCommands** via `minijinja`, replacing ad-hoc `str::replace("{{args}}")`.
- **Maintain backward compatibility**: existing templates using `{{args}}` keep working.
- **Add a small, explicit template context allowlist** for SlashCommands (e.g., `args`, `argv`, `command`, `cwd`, `workspace_root`).
- **Add tagged system prompt sections** (lightweight XML-like markers) to make the prompt easier to scan.
- **Improve error handling**: template syntax errors are reported with file/name context; runtime render errors do not crash the agent.

## Impact

### Affected Specs
- `specs/agent-core` - Modify "Dynamic System Prompt" and "System Prompt Construction" requirements
- `specs/slash-commands` - Modify command template expansion requirements

### Affected Code
- `crates/core/src/agent/prompt.rs` - SystemPromptBuilder to generate XML structure
- `crates/core/src/agent.rs` - Template expansion logic (replace `str::replace` with minijinja)
- `crates/core/src/command/loader.rs` - Command template loading with Jinja2 validation
- `crates/core/Cargo.toml` - Add `minijinja` dependency

### Breaking Changes
None. Existing `{{args}}` syntax will continue to work; new features are opt-in.

### Migration Path
- No migration required for existing commands (backward compatible)
- New templates can leverage Jinja2 features immediately
- Migration examples will be included in this change’s proposal/design text (no new docs in this change).
