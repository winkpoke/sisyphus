## 1. Template Engine Extension
- [x] 1.1 Add `SystemPromptContext::capture()` method to `crates/core/src/template.rs`
- [x] 1.2 Add `get_default_system_prompt_template()` function to `crates/core/src/template.rs` that uses `include_str!` to load default template from file
- [x] 1.3 Export `SystemPromptContext` from `crates/core/src/template.rs`

## 2. System Prompt Template
- [x] 2.1 Create default system prompt template file at `crates/core/src/agent/templates/system_prompt.jinja`
- [x] 2.2 Include template variables: `{{instructions}}`, `{{custom_rules}}`, `{{os}}`, `{{cwd}}`, `{{date}}`, `{{workspace_root}}`
- [x] 2.3 Use XML-style tags for semantic sections (`<instructions>`, `<environment>`, `<project_rules>`)

## 3. PromptSnapshot Removal
- [x] 3.1 Remove `PromptSnapshot` struct from `crates/core/src/agent/prompt.rs`
- [x] 3.2 Remove `PromptSnapshot` imports from `crates/core/src/agent/prompt.rs`

## 4. Configuration Extension
- [x] 4.1 Add `system_prompt_template` field to `AgentConfig` in `crates/core/src/agent/config.rs`
- [x] 4.2 Make `system_prompt_template` optional with default of `None`
- [x] 4.3 Update `AgentConfig::default()` to include new field

## 5. Builder Refactor
- [x] 5.1 Refactor `SystemPromptBuilder::new()` to load default template
- [x] 5.2 Add `SystemPromptBuilder::new_with_template(template: &str) -> Self` constructor
- [x] 5.3 Refactor `SystemPromptBuilder::build()` to accept `workspace_root` instead of `PromptSnapshot`
- [x] 5.4 Refactor `SystemPromptBuilder::build()` to call `SystemPromptContext::capture()` and use `TemplateEngine::render()`
- [x] 5.5 Handle template rendering errors and log them clearly
- [x] 5.6 Fall back to default template if custom template fails to load or render

## 6. Template Discovery
- [x] 6.1 Support loading custom templates from `.sisyphus/templates/system_prompt.jinja`
- [x] 6.2 Add warning when custom template file exists but has syntax errors
- [x] 6.3 Add validation for template at load time (not just render time)

## 7. Tests
- [x] 7.1 Update `test_prompt_generation()` to use `SystemPromptContext` instead of `PromptSnapshot`
- [x] 7.2 Add test for custom template override
- [x] 7.3 Add test for template syntax error handling
- [x] 7.4 Add test for XML-style tag sections in rendered output
- [x] 7.5 Add test for undefined variable handling in system prompts
- [x] 7.6 Verify backward compatibility (no template = old behavior)

## 8. Performance & Safety Improvements
- [x] 9.1 Refactor `TemplateEngine` to use persistent `Arc<Environment>` for performance
- [x] 9.2 Refactor `SystemPromptContext::capture` to accept `custom_rules` argument (removing internal I/O)
- [x] 9.3 Update `Agent::run_turn_loop` to perform async file I/O for `AGENTS.md` before capturing context
- [x] 9.4 Update `SystemPromptBuilder` to return `Result<String>` instead of swallowing errors
- [x] 9.5 Add consistent XML escaping for all injected template variables
- [x] 9.6 Fix `CommandLoader` validation to respect frontmatter
