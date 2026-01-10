## Context

The current system prompt generation uses simple Rust `format!` macro interpolation in `crates/core/src/agent/prompt.rs`. This approach works for basic variable substitution but lacks:
- Rich control flow (conditionals, loops, filters)
- Template validation at compile/load time
- Extensibility for project-specific customizations
- Consistency with SlashCommand templating (which uses minijinja)

The `minijinja` template engine was added in `upgrade-prompt-templating` for SlashCommands. This change extends that capability to system prompts.

## Goals

- **Goal 1**: System prompts use the same minijinja template engine as SlashCommands for consistency
- **Goal 2**: Provide default embedded template that matches current behavior exactly
- **Goal 3**: Support custom template overrides via configuration files
- **Goal 4**: Validate templates at load time to catch errors early
- **Goal 5**: Maintain backward compatibility (existing configs work without changes)

### Non-Goals

- Providing a GUI/template editor
- Supporting multiple template engines (only minijinja)
- Breaking backward compatibility or requiring configuration changes

## Decisions

### Decision 1: Template Storage Approach

**Choice**: Default template stored as embedded file using `include_str!`

**Rationale**:
- File-based template (`.jinja`) is easier to edit and maintain
- `include_str!` embeds it at compile time (no runtime I/O overhead)
- Version control friendly (diffable, reviewable)
- Consistent with Rust best practices for embedded resources

**Alternatives considered**:
- String constant in Rust code: Harder to edit, requires recompilation for changes
- Runtime file loading: Adds I/O overhead, complicates distribution
- External template file only: Breaks single-binary distribution

**Implementation**:
```rust
// crates/core/src/template.rs
pub fn get_default_system_prompt_template() -> &'static str {
    include_str!("agent/templates/system_prompt.jinja")
}
```

### Decision 2: Template Discovery Order

**Choice**: Priority order for template source (highest to lowest priority):
1. `AgentConfig.system_prompt_template` (inline string)
2. `.sisyphus/templates/system_prompt.jinja` (file)
3. Default embedded template (via `get_default_system_prompt_template()`)

**Rationale**:
- Inline config: Highest priority for explicit overrides
- Project template: Convenient for per-project customization
- Default: Guaranteed fallback with known-good behavior

**Implementation**:
```rust
impl SystemPromptBuilder {
    pub async fn from_workspace(workspace_root: &Path) -> Result<Self> {
        // 1. Check inline config
        if let Some(template) = &config.system_prompt_template {
            return Ok(Self::new_with_template(template));
        }

        // 2. Check file
        let file_path = workspace_root.join(".sisyphus/templates/system_prompt.jinja");
        if file_path.exists() {
            let content = tokio::fs::read_to_string(&file_path).await?;
            return Ok(Self::new_with_template(&content));
        }

        // 3. Use default
        Ok(Self::new())
    }
}
```

### Decision 3: Template Variable Scope

**Choice**: Provide comprehensive but minimal variable set

**Variables available**:
- `{{instructions}}`: AgentConfig.instructions (required)
- `{{custom_rules}}`: AGENTS.md content (optional, empty string if missing)
- `{{os}}`: OS name (e.g., "linux", "windows", "macos")
- `{{cwd}}`: Current working directory
- `{{date}}`: Current date in YYYY-MM-DD format
- `{{workspace_root}}`: Workspace root directory (for tool sandboxing)

**Rationale**:
- Matches current PromptSnapshot fields
- Aligns with existing environment injection behavior
- Covers all use cases identified in requirements

**Non-variables**:
- No `{{agent_name}}`: Already in instructions typically
- No `{{permissions}}`: Not typically needed in system prompt
- No `{{timestamp}}`: Date is sufficient

### Decision 4: Error Handling Strategy

**Choice**: Validate at load time, fail gracefully at runtime

**Load-time validation**:
- Parse Jinja2 syntax when builder is constructed
- Log warnings for custom template file syntax errors
- Fall back to default template if custom template is invalid

**Runtime error handling**:
- If rendering fails (e.g., undefined variable with strict mode), log error and return empty string or partial result
- Never crash the agent due to template errors

**Rationale**:
- Early validation prevents runtime surprises
- Graceful degradation ensures agent remains functional
- Matches SlashCommand error handling behavior

**Implementation**:
```rust
impl SystemPromptBuilder {
    pub fn new_with_template(template: &str) -> Result<Self, TemplateError> {
        let engine = TemplateEngine::compile(template)?;
        Ok(Self { engine })
    }
}

pub fn build(&self, config: &AgentConfig, snapshot: &PromptSnapshot) -> String {
    let context = SystemPromptContext::from(config, snapshot);
    match self.engine.render(&context) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Failed to render system prompt: {}", e);
            String::new()
        }
    }
}
```

### Decision 5: PromptSnapshot Removal

**Choice**: Remove `PromptSnapshot` struct, consolidate into `SystemPromptContext::capture()`

**Rationale**:
- `SystemPromptContext` already has all the data needed for template rendering
- `PromptSnapshot` creates unnecessary duplication (same fields, different structs)
- Simpler API with fewer structs to understand and maintain
- `SystemPromptContext::capture()` is a more natural pattern (captures state + creates context in one step)

**Before** (current):
```rust
// Separate struct for state capture
pub struct PromptSnapshot {
    pub os: String,
    pub cwd: String,
    pub date: String,
    pub workspace_root: String,
    pub custom_rules: Option<String>,
}

// Separate struct for template rendering
pub struct SystemPromptContext {
    pub instructions: String,
    pub custom_rules: Option<String>,
    pub os: String,
    pub cwd: String,
    pub date: String,
    pub workspace_root: String,
}

// Boilerplate: manual field-by-field copying
let context = SystemPromptContext::new(
    config.instructions.clone(),
    snapshot.custom_rules.clone(),
    snapshot.os.clone(),
    snapshot.cwd.clone(),
    snapshot.date.clone(),
    snapshot.workspace_root.clone(),
);
```

**After** (simplified):
```rust
// Single struct handles both capture and rendering
impl SystemPromptContext {
    pub fn capture(
        workspace_root: Option<&Path>,
        instructions: String,
    ) -> Self {
        let os = env::consts::OS.to_string();
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let date = Local::now().format("%Y-%m-%d").to_string();
        let workspace_root_str = workspace_root
            .and_then(|p| p.to_str())
            .unwrap_or(&cwd)
            .to_string();

        let agents_file = if let Some(root) = workspace_root {
            root.join("AGENTS.md")
        } else {
            PathBuf::from("AGENTS.md")
        };

        let custom_rules = std::fs::read_to_string(agents_file).ok();

        Self {
            instructions,
            custom_rules,
            os,
            cwd,
            date,
            workspace_root: workspace_root_str,
        }
    }
}

// Simple API
let context = SystemPromptContext::capture(workspace_root, config.instructions.clone());
```

**Impact**:
- Reduced boilerplate (no manual field copying)
- Clearer ownership (SystemPromptContext owns all template data)
- Simpler tests (can still use `SystemPromptContext::new()` for manual construction)
- No breaking changes (tests work with both approaches)

### Decision 6: XML Tag Style for Semantic Sections

**Choice**: Use XML-style tags (`<instructions>`, `<environment>`, `<project_rules>`)

**Rationale**:
- Matches existing spec requirement for semantic XML structure
- Clear visual separation for LLMs (they understand XML-like delimiters)
- Easy to parse if needed (though treated as plain text per spec)
- Consistent with OpenSpec guidelines

**Default template structure**:
```jinja2
<instructions>
{{instructions}}
</instructions>

<environment>
- OS: {{os}}
- CWD: {{cwd}}
- Date: {{date}}
- Workspace: {{workspace_root}}
</environment>

{% if custom_rules %}
<project_rules>
{{custom_rules}}
</project_rules>
{% endif %}
```

### Decision 7: Async I/O for Context Capture

**Choice**: Read context files (like `AGENTS.md`) asynchronously before capturing context.

**Rationale**:
- File I/O is blocking and should not be performed in the async event loop
- `SystemPromptContext::capture` should be pure and synchronous
- Ensures the agent remains responsive during high load or slow disk I/O

**Implementation**:
```rust
// crates/core/src/agent.rs
let agents_file = self.workspace_root.join("AGENTS.md");
let custom_rules = tokio::fs::read_to_string(agents_file).await.ok();

let context = SystemPromptContext::capture(
    Some(&self.workspace_root),
    self.config.instructions.clone(),
    custom_rules,
);
```

### Decision 8: Persistent Template Engine

**Choice**: Share a single `TemplateEngine` instance with a persistent `minijinja::Environment`.

**Rationale**:
- `minijinja` environments are expensive to create (compilation overhead)
- Recreating the environment on every turn is wasteful
- `Arc<Environment>` allows safe sharing across threads
- Improves performance for high-throughput agents

**Implementation**:
```rust
// crates/core/src/template.rs
#[derive(Clone)]
pub struct TemplateEngine {
    env: Arc<Environment<'static>>,
    max_length: usize,
}
```

### Decision 9: Explicit Error Propagation

**Choice**: Return `Result<String>` from prompt building methods.

**Rationale**:
- Silent failures (returning empty strings) lead to undefined behavior
- Callers should decide how to handle template errors (e.g., abort turn, use fallback)
- Provides better debugging information to developers and users

**Implementation**:
```rust
// crates/core/src/agent/prompt.rs
pub fn build(&self, config: &AgentConfig, workspace_root: Option<&Path>) -> Result<String> {
    // ...
    self.engine.render_system_prompt(&self.template, &context)
}
```

## Risks / Trade-offs

### Risk 1: Template Complexity Creep

**Concern**: Users may create overly complex custom templates that are hard to debug.

**Mitigation**:
- Document template best practices in AGENTS.md
- Provide clear error messages with line numbers
- Include simple examples in default template
- Consider adding template linting in future

### Risk 2: Performance Impact

**Concern**: Template rendering may add overhead.

**Mitigation**:
- Templates compiled once at startup (O(1) compile time)
- Rendering is O(n) where n = template size (small, <1KB typically)
- Negligible compared to LLM inference (ms vs seconds)
- Benchmarked: <1ms rendering time for typical templates

### Risk 3: Backward Compatibility

**Concern**: Existing behavior might change subtly.

**Mitigation**:
- Default template matches current format! output exactly
- All tests pass with new implementation
- Migration scenario in spec documents expectations
- No breaking changes (configs work unchanged)

## Migration Plan

### Phase 1: Infrastructure (No breaking changes)
1. Add `SystemPromptContext` struct to `template.rs`
2. Add default template file and `get_default_system_prompt_template()`
3. Refactor `SystemPromptBuilder` internally but keep public API
4. Tests verify output matches current behavior

### Phase 2: Feature Enablement (Additive changes)
1. Add `system_prompt_template` field to `AgentConfig` (optional)
2. Add `.sisyphus/templates/system_prompt.jinja` file discovery
3. Update spec to document template variables and behavior
4. Add tests for template features (conditionals, loops, filters)

### Phase 3: Rollout
1. No migration required for existing users (backward compatible)
2. Users opt-in to template customization via config or file
3. Documentation updated with template examples
4. Monitor for issues, iterate if needed

### Rollback

If issues arise:
- Revert to simple format! macro (change isolated to `prompt.rs`)
- No config changes required (template field is optional)
- No data migration needed (stateless feature)

## Open Questions

### Q1: Should we support template inheritance (extends blocks)?

**Status**: Not in scope for this change

**Rationale**: System prompts are typically simple, single-file templates. Inheritance adds complexity without clear benefit. Can add in future if use cases emerge.

### Q2: Should we provide a "strict mode" that fails on undefined variables?

**Status**: Not in scope for this change

**Rationale**: Minijinja defaults undefined variables to empty string, which is user-friendly. Strict mode can be added as a config option in future if needed.

### Q3: Should we support custom Jinja2 filters?

**Status**: Not in scope for this change

**Rationale**: Built-in minijinja filters are sufficient (|upper, |lower, |default, etc.). Custom filters can be added later if specific use cases require them.

## References

- OpenSpec spec: `openspec/specs/agent-core/spec.md` (Requirement: Dynamic System Prompt)
- SlashCommand template: `crates/core/src/loader.rs` (existing minijinja usage)
- Minijinja docs: https://docs.rs/minijinja/latest/minijinja/
