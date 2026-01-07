## 1. SlashCommand templating
- [ ] 1.1 Add `minijinja` dependency to `crates/core/Cargo.toml`
- [ ] 1.2 Validate custom command templates for syntax during load (per-file warn+skip)
- [ ] 1.3 Render custom commands using an allowlisted template context (`args`, `argv`, `command`, `cwd`, `workspace_root`)
- [ ] 1.4 Enforce a max rendered length for expanded SlashCommands
- [ ] 1.5 Keep existing recursion limit for chained command expansion

## 2. System prompt structure
- [ ] 2.1 Wrap system prompt content in tagged sections (`<instructions>`, `<environment>`, `<project_rules>`)
- [ ] 2.2 Escape `AGENTS.md` content embedded in `<project_rules>`
- [ ] 2.3 Update unit tests for `SystemPromptBuilder` to match the new structure

## 3. Validation
- [ ] 3.1 Run `cargo test -p sisyphus-core`
- [ ] 3.2 Run `openspec validate upgrade-prompt-templating --strict`
