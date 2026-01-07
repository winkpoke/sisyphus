# Change: Upgrade Prompt Templating with Jinja2 and XML Structure

## Why

The current prompt system uses simple string replacement (`{{args}}`) which limits template flexibility and maintainability. Additionally, system prompts lack semantic structure, making them harder to understand, parse, and optimize.

Modern best practices (2024-2025) recommend:
1. **Jinja2-style templating** for rich variable interpolation, loops, and conditionals
2. **XML-tagged structure** (Anthropic-recommended) for semantic chunking that improves model understanding

This upgrade will:
- Enable maintainable, reusable templates (DRY principles)
- Support complex template logic (loops, conditionals, nested objects)
- Align with industry best practices for prompt engineering
- Improve prompt clarity through semantic structure

## What Changes

- **Add Jinja2 template engine** (`minijinja` crate) for SlashCommand and system prompt templates
- **Introduce XML-structured system prompts** with semantic tags (`<role>`, `<task>`, `<instructions>`, `<environment>`, `<project_rules>`, `<output_format>`)
- **Maintain backward compatibility** with existing `{{args}}` syntax for custom commands
- **Extend template variables** beyond `{{args}}` to include session context, environment, and user metadata
- **Improve error handling** with clear template parsing and validation errors

## Impact

### Affected Specs
- `specs/agent-core` - Modify "Dynamic System Prompt" and "System Prompt Construction" requirements
- `specs/slash-commands` - Modify command template expansion requirements

### Affected Code
- `crates/core/src/agent/prompt.rs` - SystemPromptBuilder to generate XML structure
- `crates/core/src/agent.rs` - Template expansion logic (replace `str::replace` with minijinja)
- `crates/core/src/command/loader.rs` - Command template loading with Jinja2 validation
- `crates/cli/src/Cargo.toml` - Add `minijinja` dependency

### Breaking Changes
None. Existing `{{args}}` syntax will continue to work; new Jinja2 features are opt-in.

### Migration Path
- No migration required for existing commands (backward compatible)
- New templates can leverage Jinja2 features immediately
- Documentation will provide migration examples for templates
