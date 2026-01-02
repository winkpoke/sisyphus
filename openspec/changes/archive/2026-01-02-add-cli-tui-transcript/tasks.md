## 1. Implementation
- [x] 1.1 Define typed UI events that represent streamed and discrete transcript updates
- [x] 1.2 Implement transcript state model (cells/rows) with append and in-place update
- [x] 1.3 Implement scrolling behavior (stick-to-bottom, manual scroll, jump-to-bottom)
- [x] 1.4 Implement resizing behavior with correct wrapping/reflow
- [x] 1.5 Ensure the input composer remains responsive during heavy streaming

## 2. Validation
- [x] 2.1 Add snapshot tests for transcript rendering across widths
- [x] 2.2 Add tests for scrolling invariants (stick-to-bottom vs manual scroll)
- [x] 2.3 Run formatting, lint, and test suite (cargo fmt, cargo clippy, cargo test)
