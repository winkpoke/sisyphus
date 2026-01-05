## 1. Command semantics
- [ ] 1.1 Define UiCommand vs SlashCommand types and routing rules
- [ ] 1.2 Define reserved UiCommand names and collision behavior with custom SlashCommands

## 2. Server API
- [ ] 2.1 Add global SlashCommand discovery endpoint returning metadata only
- [ ] 2.2 Add session history clear endpoint suitable for UiCommand `/clear`
- [ ] 2.3 Keep chat endpoint focused on chat turns and SlashCommand expansion only

## 3. Client behavior
- [ ] 3.1 Implement client-side command router with UiCommand precedence
- [ ] 3.2 Update command palette to show merged UiCommand + SlashCommand list
- [ ] 3.3 Implement UiCommand `/help` output or overlay based on merged list

## 4. Validation
- [ ] 4.1 Update and run tests for command routing and discovery behavior
- [ ] 4.2 Run `openspec validate refactor-command-system-ui-slash --strict`
