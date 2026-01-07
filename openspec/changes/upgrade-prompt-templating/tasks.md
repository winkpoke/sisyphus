## 1. Foundation (Week 1)

### 1.1 Add minijinja dependency
- [ ] Add `minijinja = { version = "2.0", features = ["loader"] }` to `crates/core/Cargo.toml`
- [ ] Verify dependency compiles with `cargo build`
- [ ] Add `minijinja` to `crates/cli/Cargo.toml` (if needed for CLI binary)

### 1.2 Create TemplateEngine wrapper
- [ ] Create `crates/core/src/template/mod.rs` module
- [ ] Create `crates/core/src/template/engine.rs` with `TemplateEngine` struct
- [ ] Implement `TemplateEngine::new()` constructor
- [ ] Implement `TemplateEngine::compile(template_str: &str) -> Result<CompiledTemplate, TemplateError>`
- [ ] Implement `TemplateEngine::render(compiled: &CompiledTemplate, variables: &Value) -> Result<String, TemplateError>`
- [ ] Add `TemplateError` enum with clear error variants (ParseError, RenderError, UndefinedVariable)
- [ ] Implement `Display` trait for `TemplateError` (user-friendly messages)

### 1.3 Implement basic variable interpolation
- [ ] Add unit test for `{{args}}` replacement (backward compatibility)
- [ ] Add unit test for multiple variables (e.g., `{{var1}} {{var2}}`)
- [ ] Add unit test for nested objects (e.g., `{{user.name}}`)
- [ ] Add unit test for array access (e.g., `{{files[0]}}`)

### 1.4 Add validation and error handling
- [ ] Add unit test for undefined variable error
- [ ] Add unit test for syntax error (e.g., unclosed bracket)
- [ ] Verify error messages include template source context (file name, line number)
- [ ] Add integration test for template compilation failure handling

## 2. System Prompts (Week 2)

### 2.1 Design XML structure for system prompts
- [ ] Create design document listing required XML tags: `<role>`, `<task>`, `<instructions>`, `<environment>`, `<project_rules>`, `<output_format>`
- [ ] Define content for each tag (what information goes where)
- [ ] Document tag nesting rules (if any)

### 2.2 Modify SystemPromptBuilder
- [ ] Update `SystemPromptBuilder::build()` to generate XML structure
- [ ] Implement XML tag wrapping: `wrap_xml_section(section_name: &str, content: &str) -> String`
- [ ] Add `<role>` tag with agent description/persona
- [ ] Add `<task>` tag with task description from AgentConfig
- [ ] Add `<instructions>` tag with `config.instructions`
- [ ] Add `<environment>` tag with OS, CWD, Date (existing logic)
- [ ] Add `<project_rules>` tag for AGENTS.md content (conditional)
- [ ] Add `<output_format>` tag with default response format guidelines

### 2.3 Update environment snapshot logic
- [ ] Ensure `PromptSnapshot` captures all needed fields
- [ ] Verify XML tags use consistent formatting (indentation, line breaks)
- [ ] Add unit test for XML structure generation

### 2.4 Update existing tests
- [ ] Find all tests in `crates/core/src/agent/prompt.rs`
- [ ] Update tests to expect XML structure instead of flat prompts
- [ ] Verify all tests pass with `cargo test prompt`
- [ ] Add integration test for full system prompt generation

### 2.5 Verify LLM compatibility
- [ ] Test XML structure with actual LLM provider (OpenAI)
- [ ] Verify LLM understands and respects XML tags
- [ ] Check that prompts don't break model behavior
- [ ] Add test case for XML tag escaping (code blocks with `<` and `>`)

## 3. Command Templates (Week 3)

### 3.1 Update CommandLoader validation
- [ ] Modify `CommandLoader::load_from_dir()` to validate templates with Jinja2
- [ ] Add template parsing to frontmatter extraction logic
- [ ] Store `CompiledTemplate` in `CommandConfig` (not raw string)
- [ ] Return clear error message for invalid templates (include file name, line, context)
- [ ] Add unit test for valid template loading
- [ ] Add unit test for invalid template rejection (syntax error, undefined variable)

### 3.2 Replace string replacement in agent runtime
- [ ] Locate template expansion in `crates/core/src/agent.rs` (around lines 203-207)
- [ ] Replace `str::replace("{{args}}")` with `TemplateEngine::render()`
- [ ] Pass `args` variable to template rendering context
- [ ] Add unit test for template rendering with `{{args}}` variable
- [ ] Add unit test for template rendering with additional variables (if exposed)

### 3.3 Add runtime error handling
- [ ] Handle `TemplateError` in command execution
- [ ] Log template rendering failures with clear context
- [ ] Return user-friendly error message to agent
- [ ] Add integration test for runtime template error

### 3.4 Update slash-commands tests
- [ ] Find all tests in `crates/core/src/command/`
- [ ] Update tests to use Jinja2 templates instead of string replacement
- [ ] Verify all tests pass with `cargo test command`
- [ ] Add test for Jinja2 features (conditional, loop, filter)

### 3.5 Add template examples
- [ ] Create example commands demonstrating Jinja2 features:
  - Simple variable: `{{args}}`
  - Multiple variables: `{{user}} {{command}}`
  - Conditional: `{% if has_permission %}...{% endif %}`
  - Loop: `{% for item in items %}...{% endfor %}`
  - Filter: `{{args|upper}}`
- [ ] Place examples in `.opencode/command/` directory
- [ ] Verify examples load and render correctly

## 4. Documentation (Week 4)

### 4.1 Write template syntax guide
- [ ] Create `docs/template-syntax.md` with:
  - Variable interpolation (`{{var}}`)
  - Accessing nested objects (`{{user.name}}`)
  - Loops (`{% for %}`)
  - Conditionals (`{% if %}`)
  - Filters (`{{var|upper}}`)
  - Template inheritance (if implemented)
  - Custom filters (if implemented)
- [ ] Add code examples for each feature
- [ ] Link to Jinja2 official documentation for advanced usage

### 4.2 Create migration guide
- [ ] Document how to migrate existing `{{args}}` templates to Jinja2
- [ ] Provide before/after examples
- [ ] List common pitfalls and how to avoid them
- [ ] Document backward compatibility guarantees

### 4.3 Document XML tag structure
- [ ] Create `docs/system-prompts.md` with:
  - Purpose of each XML tag
  - When to use each tag
  - Best practices for tag content
  - Example of complete system prompt
- [ ] Document tag ordering and nesting rules
- [ ] Explain model benefits of XML structure

### 4.4 Add troubleshooting section
- [ ] Document common template errors:
  - Undefined variable
  - Syntax error
  - Missing closing tag
  - Filter not found
- [ ] Provide solutions for each error
- [ ] Add debugging tips (how to inspect compiled templates)

### 4.5 Update AGENTS.md
- [ ] Add section on prompt templating to project-specific rules
- [ ] Document custom template conventions (if any)
- [ ] Provide examples for users to follow

## 5. Validation and Testing

### 5.1 Run existing test suite
- [ ] Run `cargo test --all` to verify no regressions
- [ ] Fix any failing tests
- [ ] Document test changes if behavior intentionally changed

### 5.2 Performance benchmarking
- [ ] Benchmark template compilation time (should be <1ms per template)
- [ ] Benchmark template rendering time (should be <1ms per render)
- [ ] Compare with old string replacement performance
- [ ] Document performance characteristics

### 5.3 Integration testing
- [ ] Test full agent chat flow with new system prompts
- [ ] Test command execution with Jinja2 templates
- [ ] Test error handling for invalid templates
- [ ] Test XML structure with actual LLM provider

### 5.4 Manual testing
- [ ] Manually test custom SlashCommands with Jinja2 features
- [ ] Verify system prompts look correct in debug mode
- [ ] Test error messages are user-friendly
- [ ] Verify backward compatibility with existing commands

## 6. Review and Approval

### 6.1 Code review checklist
- [ ] All new code has unit tests
- [ ] All tests pass (`cargo test --all`)
- [ ] Code follows project style guidelines
- [ ] Error messages are clear and actionable
- [ ] No unused dependencies added

### 6.2 Documentation review
- [ ] Template syntax guide is complete and accurate
- [ ] Migration guide is clear and helpful
- [ ] XML structure is well-documented
- [ ] Troubleshooting section covers common issues

### 6.3 Validation
- [ ] Run `openspec validate upgrade-prompt-templating --strict`
- [ ] Fix all validation errors
- [ ] Verify spec deltas are correct

### 6.4 Approval
- [ ] Proposal reviewed by stakeholders
- [ ] Design decisions approved
- [ ] Tasks estimated and prioritized
- [ ] Ready for implementation phase

## Notes

### Parallelization Opportunities
- Phase 1 (Foundation) and Phase 2 (System Prompts) can be started in parallel after 1.1 is complete
- Phase 3 (Command Templates) depends on Phase 1 completion
- Phase 4 (Documentation) can be started in parallel with Phase 3

### Dependencies
- Task 2.1 depends on Task 1.1 (minijinja dependency)
- Task 3.1 depends on Task 1.2 (TemplateEngine wrapper)
- Task 3.2 depends on Task 1.4 (validation implementation)
- Task 4.1 depends on Task 2.5 (LLM compatibility verification)

### Estimated Effort
- Phase 1: 8-12 hours
- Phase 2: 12-16 hours
- Phase 3: 10-14 hours
- Phase 4: 8-10 hours
- Phase 5: 4-6 hours
- Phase 6: 2-4 hours
- **Total**: 44-62 hours (1.5-2 weeks for full-time developer)
