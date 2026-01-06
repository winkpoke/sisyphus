## 1. Core and API cleanup
- [x] 1.1 Remove `CommandEffect` from core command outcomes and chat service
- [x] 1.2 Remove `effect` metadata from server chat response DTO

## 2. Client UiCommand alignment
- [x] 2.1 Ensure `/new` and `/clear` use explicit session endpoints
- [x] 2.2 Ensure `/exit` and `/debug` remain local UiCommands and are not sent to chat

## 3. Validation
- [x] 3.1 Update/add tests for chat response schema (no `effect`)
- [x] 3.2 Update/add tests for clear and create session endpoints
- [x] 3.3 Run `cargo test`, `cargo fmt`, and `cargo clippy`
- [x] 3.4 Run `openspec validate remove-command-effect --strict`
