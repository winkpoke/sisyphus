## 1. Implementation
- [ ] Parse backend SSE messages into `SystemEvent` in the TUI
- [ ] Render end-user summaries for supported event types
- [ ] Add local `/debug` command and include it in the command palette
- [ ] Redact and truncate raw event payload output in debug mode

## 2. Tests
- [ ] Add unit tests for event parsing and rendering mapping
- [ ] Add unit tests for redaction and truncation of raw payloads

## 3. Validation
- [ ] Run the full Rust test suite
- [ ] Run formatting and linting for Rust workspace
- [ ] Manually verify in TUI: permission overlay, tool execution event, debug toggling
