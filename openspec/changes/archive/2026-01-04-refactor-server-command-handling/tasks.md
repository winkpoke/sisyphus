## 1. Core orchestration
- [x] 1.1 Add a core chat application service to handle chat requests
- [x] 1.2 Centralize `CommandEffect` application in the core service

## 2. Server integration
- [x] 2.1 Refactor HTTP chat endpoint to delegate to the core service
- [x] 2.2 Remove `CommandEffect` handling logic from server HTTP handlers

## 3. API contract and semantics
- [x] 3.1 Extend chat response to include explicit command-effect metadata
- [x] 3.2 Clarify `/exit` as session/client-scoped, not process shutdown
- [x] 3.3 Implement `/quit` as an alias to `/exit` in the registry

## 4. Client alignment
- [x] 4.1 Update CLI REPL to rely on server command handling for `/exit` and `/quit`
- [x] 4.2 Update TUI to stop executing slash commands locally and rely on server responses

## 5. Validation
- [x] 5.1 Add/adjust tests for server chat command effects and response metadata
- [x] 5.2 Run full test suite and fix regressions
