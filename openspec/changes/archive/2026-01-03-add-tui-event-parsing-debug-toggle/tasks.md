## 1. Implementation
- [x] Parse backend SSE messages into `SystemEvent` in the TUI
- [x] Render end-user summaries for supported event types
- [x] Add local `/debug` command and include it in the command palette
- [x] Redact and truncate raw event payload output in debug mode

## 2. Tests
- [x] Add unit tests for event parsing and rendering mapping
- [x] Add unit tests for redaction and truncation of raw payloads

## 3. Validation
- [x] Run the full Rust test suite
- [x] Run formatting and linting for Rust workspace
- [x] Manually verify in TUI: permission overlay, tool execution event, debug toggling
