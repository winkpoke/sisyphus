## Context

The Sisyphus prompt system currently uses simple string concatenation and replacement:
- **System prompts**: Join `config.instructions` + `env_info` + `AGENTS.md` with `\n\n`
- **Command templates**: Replace `{{args}}` with raw argument string using `str::replace()`

This approach has limitations:
1. **No template reuse** - Common patterns must be repeated across templates
2. **No conditional logic** - Cannot adapt prompts based on context (e.g., user role, permissions)
3. **No loops** - Cannot iterate over lists (e.g., available tools, file paths)
4. **Poor error messages** - String replacement failures are cryptic
5. **No semantic structure** - Flat prompts lack clear section boundaries

Industry best practices (2024-2025) have evolved:
- **Anthropic** recommends XML-tagged prompts for semantic chunking
- **Microsoft POML** uses HTML-like structure for prompt organization
- **Production systems** (LangChain, BeeAI, Pydantic AI) use template engines (Jinja2, Handlebars)
- **Context engineering** has replaced "prompt engineering" as the paradigm

### Constraints
- Must maintain backward compatibility with existing custom SlashCommands
- Cannot introduce blocking I/O in prompt construction (existing requirement)
- Must support existing `{{args}}` syntax
- Should not significantly increase compilation time or binary size

### Stakeholders
- **Users**: Need improved prompts without breaking existing workflows
- **Developers**: Need maintainable template system with good error messages
- **System architects**: Need alignment with industry best practices

## Goals / Non-Goals

### Goals
1. **Improve SlashCommand templating** - Support Jinja2 variables/filters for custom commands.
2. **Add semantic structure** - Use lightweight XML-like tags for system prompt sections.
3. **Maintain backward compatibility** - Existing templates using `{{args}}` continue to work.
4. **Keep scope small** - Avoid large refactors (no async rewrite of command loading).
5. **Improve developer experience** - Clear, contextual template errors.

### Non-Goals
1. **Prompt versioning** - Out of scope (suggested in medium priority roadmap)
2. **Prompt caching** - Out of scope (suggested in medium priority roadmap)
3. **Dynamic middleware** - Out of scope (suggested in medium priority roadmap)
4. **Strict XML semantics** - Tags are markers; output is not required to be well-formed XML.
5. **Web-based prompt editor** - Out of scope (suggested in low priority roadmap)

## Decisions

### Decision 1: Use minijinja crate over Handlebars/Tera
**What**: Adopt `minijinja` (Jinja2-compatible template engine for Rust)

**Why**:
- Jinja2 is industry standard (Python, web frameworks, production systems)
- `minijinja` has excellent Rust integration and performance
- Familiar syntax for developers with web/Python background
- Rich feature set: filters, tests, template inheritance, macros
- Active maintenance and good documentation

**Alternatives considered**:
- **Handlebars**: Logic-less by design, limited conditionals
- **Tera**: Django-inspired, less familiar than Jinja2
- **Askama**: Compile-time only, not suitable for dynamic templates
- **Custom string interpolation**: Reimplementing Jinja2 features (too complex)

**Trade-offs**:
- Adds ~0.5MB to binary size (acceptable)
- Minor compilation time increase (acceptable)
- Learning curve for template syntax (mitigated by examples and documentation)

### Decision 2: XML Structure Over Markdown Headers
**What**: Use lightweight XML-like section tags (e.g., `<instructions>`, `<environment>`) instead of Markdown headers.

**Why**:
- **Semantic nesting**: XML supports nested structure better than flat Markdown
- **Model understanding**: LLMs process XML tags more reliably than arbitrary headers
- **Alignment**: Matches Anthropic's recommended pattern and POML
- **Parseability**: Easier to extract/modify sections programmatically
- **Clear boundaries**: Opening/closing tags are unambiguous

**Alternatives considered**:
- **Markdown headers**: Too flat, no nesting, ambiguous (what is "## Task"?)
- **Custom delimiters**: `[[role]]`, `---TASK---` (non-standard, confusing)
- **JSON structure**: Verbose, not human-readable

**Trade-offs**:
- Slightly more verbose than Markdown
- We treat tags as markers (not strict XML), so escaping rules must be explicit for injected content

### Decision 3: Backward Compatibility Layer
**What**: Existing `{{args}}` syntax continues to work via Jinja2 compatibility

**Why**:
- Zero-downtime migration for existing custom commands
- No breaking changes for users
- Gradual adoption path (old templates work, new templates use features)

**Implementation**:
- Jinja2 natively supports `{{args}}` syntax
- No special handling required
- Old commands get Jinja2 expansion automatically

**Trade-offs**:
- Cannot deprecate `{{args}}` later (must support indefinitely)
- Mix of old and new template styles in wild

### Decision 4: Template Validation at Load Time
**What**: Validate SlashCommand templates for syntax at load time; validate variables at render time.

**Why**:
- Early error detection (fail-fast)
- Better error messages with file context
- No runtime surprises
- Clearer debugging experience

**Implementation**:
- `CommandLoader` compiles templates to catch syntax errors early (per-file).
- Invalid templates are skipped with a warning that includes file name and error summary.
- Undefined variables are treated as a render-time error (load time does not know render context).

**Trade-offs**:
- Slightly slower startup time (acceptable)
- Startup remains resilient (bad custom command does not prevent agent start)

## Risks / Trade-offs

### Risk 1: Binary Size Increase
**Risk**: Adding minijinja increases binary size by ~0.5MB

**Mitigation**:
- 0.5MB is acceptable for modern systems (Sisyphus is not embedded)
- Can enable feature flags if size becomes critical (unlikely)
- Performance gains from better templating outweigh size cost

### Risk 2: Learning Curve for Template Syntax
**Risk**: Developers unfamiliar with Jinja2 will struggle with template creation

**Mitigation**:
- Provide comprehensive documentation with examples
- Include migration guide for existing templates
- Add template validation with clear error messages
- Use simple examples for common patterns

### Risk 3: XML Tag Confusion with LLMs
**Risk**: LLMs might treat XML tags as code and attempt to parse them

**Mitigation**:
- Place XML tags in system prompt (not user messages)
- Use clear, semantic tag names that LLMs recognize
- Follow Anthropic's proven patterns (widely tested)
- Document tag usage explicitly in agent instructions

### Risk 4: Template Injection Attacks
**Risk**: Malicious templates could access unintended variables or execute logic

**Mitigation**:
- Whitelist allowed variables (not expose entire session state)
- Cap rendered output length for SlashCommands
- Keep runtime recursion limit for chained SlashCommands

### Risk 5: Performance Regression
**Risk**: Template parsing and rendering could slow down prompt construction

**Mitigation**:
- Compile templates at load time (not every request)
- Cache compiled templates in `CommandRegistry`
- Benchmark performance and optimize if needed
- Expect <1ms overhead (acceptable for prompt construction)

## Migration Plan

### Phase 1: Foundation (Week 1)
1. Add `minijinja` dependency to `crates/core/Cargo.toml`
2. Add a small template helper to compile/render templates with an allowlisted context
3. Add unit tests for template parsing and rendering

### Phase 2: System Prompts (Week 2)
1. Modify `SystemPromptBuilder` to generate XML structure
2. Add `<role>`, `<task>`, `<instructions>`, `<environment>`, `<project_rules>`, `<output_format>` tags
3. Update existing tests to match new XML format
4. Verify LLM compatibility with XML structure

### Phase 3: Command Templates (Week 3)
1. Modify `CommandLoader` to validate templates at load time
2. Replace `str::replace("{{args}}")` with `TemplateEngine::render()`
3. Add error handling for invalid templates
4. Update slash-commands tests for Jinja2 syntax

### Phase 4: Documentation (Week 4)
No new documentation files in this change. Keep examples in proposal/design text.

### Rollback Plan
If issues arise:
1. Revert to `str::replace()` in `agent.rs` (simple string replacement)
2. Remove XML tags from `SystemPromptBuilder` (use flat structure)
3. Keep minijinja dependency but disable its use
4. Document rollback path for users

## Open Questions

1. **Should we add custom Jinja2 filters for Sisyphus-specific operations?**
   - Consider: `format_date()`, `truncate()`, `escape_xml()`
   - Decision: Postpone to phase 2 (evaluate actual needs first)

2. **Should we support template inheritance for common prompt patterns?**
   - Example: Base template with `<role>`, `<task>` tags extended by agents
   - Decision: Postpone to phase 2 (complexity not justified yet)

3. **Should we cache compiled templates per-session or globally?**
   - Trade-off: Memory usage vs. performance
   - Decision: Global cache (templates don't change during runtime)

4. **Should we expose template variables from session context?**
   - Example: `{{session_id}}`, `{{user_name}}`, `{{workspace_root}}`
   - Decision: Yes, but whitelist variables explicitly (not expose entire session)

5. **How should we handle template rendering failures at runtime?**
   - Current: String replacement never fails (empty string for missing vars)
   - Proposed: Fail with error and log template issue
   - Decision: Fail with error (better to surface issues than silently produce bad prompts)
