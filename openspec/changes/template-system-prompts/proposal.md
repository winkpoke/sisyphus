# Change: Template-based System Prompts

## Why

System prompts are currently built using simple Rust `format!` macro interpolation, which lacks:
- **Rich control flow** (conditionals, loops)
- **Template validation** at load time
- **Consistency** with SlashCommand templating (which uses minijinja)
- **Extensibility** for project-specific customizations

The `minijinja` template engine was added in `upgrade-prompt-templating` for SlashCommands, but system prompts remain on simple string formatting. This creates inconsistency and limits future enhancements.

## What Changes

- Add template-based system prompts using the existing `TemplateEngine` and `minijinja`
- Provide default system prompt template with Jinja2 syntax
- Support custom system prompt template overrides via configuration
- Maintain backward compatibility with existing agent configs
- Add validation for system prompt templates at load time
- Update tests to cover new template rendering behavior

## Impact

- **Affected specs**: `agent-core` (adds/modifies system prompt construction requirements)
- **Affected code**:
  - `crates/core/src/agent/prompt.rs` - refactor `SystemPromptBuilder`, remove `PromptSnapshot`, add error propagation
  - `crates/core/src/agent/config.rs` - optional `system_prompt_template` field
  - `crates/core/src/template.rs` - extend `SystemPromptContext`, refactor `TemplateEngine` for persistence and XML escaping
  - `crates/core/src/agent.rs` - async I/O for `AGENTS.md` reading
  - `crates/core/src/command/loader.rs` - fix validation logic
- **No breaking changes**: Existing `AgentConfig.instructions` strings continue to work
- **Additive enhancement**: New `system_prompt_template` field provides template capability
- **Simplification**: Removed `PromptSnapshot` struct, consolidated into `SystemPromptContext::capture()`
