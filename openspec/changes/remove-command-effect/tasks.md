## 1. Core and API cleanup
- [ ] 1.1 Remove `CommandEffect` from core command outcomes and chat service
- [ ] 1.2 Remove `effect` metadata from server chat response DTO

## 2. Client UiCommand alignment
- [ ] 2.1 Ensure `/new` and `/clear` use explicit session endpoints
- [ ] 2.2 Ensure `/exit` and `/debug` remain local UiCommands and are not sent to chat

## 3. Validation
- [ ] 3.1 Update/add tests for chat response schema (no `effect`)
- [ ] 3.2 Update/add tests for clear and create session endpoints
- [ ] 3.3 Run `cargo test`, `cargo fmt`, and `cargo clippy`
- [ ] 3.4 Run `openspec validate remove-command-effect --strict`
