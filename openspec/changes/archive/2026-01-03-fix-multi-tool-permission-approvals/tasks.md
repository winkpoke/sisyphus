## 1. Implementation
- [x] 1.1 Update core agent tool-call loop to implement sequential blocking (stop at first Ask)
- [x] 1.2 Refactor Session/Agent to maintain "pending batch" state for resumption
- [x] 1.3 Ensure `Allow` tools in a batch are only executed if all preceding tools are resolved
- [x] 1.4 Update TUI to handle sequential PermissionRequest events (ensure overlay updates when new event arrives)

## 2. Validation
- [x] 2.1 Add a regression test for multi-tool messages with mixed Allow/Ask
- [x] 2.2 Add a regression test for multiple sequential PermissionRequest events in TUI state
- [x] 2.3 Run workspace tests and ensure no regressions
