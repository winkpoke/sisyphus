## 1. Core orchestration
- [ ] 1.1 Add a core chat application service to handle chat requests
- [ ] 1.2 Centralize `CommandEffect` application in the core service

## 2. Server integration
- [ ] 2.1 Refactor HTTP chat endpoint to delegate to the core service
- [ ] 2.2 Remove `CommandEffect` handling logic from server HTTP handlers

## 3. API contract and semantics
- [ ] 3.1 Extend chat response to include explicit command-effect metadata
- [ ] 3.2 Clarify `/exit` as session/client-scoped, not process shutdown
- [ ] 3.3 Implement `/quit` as an alias to `/exit` in the registry

## 4. Client alignment
- [ ] 4.1 Update CLI REPL to rely on server command handling for `/exit` and `/quit`
- [ ] 4.2 Update TUI to stop executing slash commands locally and rely on server responses

## 5. Validation
- [ ] 5.1 Add/adjust tests for server chat command effects and response metadata
- [ ] 5.2 Run full test suite and fix regressions
