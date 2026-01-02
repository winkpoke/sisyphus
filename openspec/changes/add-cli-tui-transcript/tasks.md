## 1. Implementation
- [ ] 1.1 Define typed UI events that represent streamed and discrete transcript updates
- [ ] 1.2 Implement transcript state model (cells/rows) with append and in-place update
- [ ] 1.3 Implement scrolling behavior (stick-to-bottom, manual scroll, jump-to-bottom)
- [ ] 1.4 Implement resizing behavior with correct wrapping/reflow
- [ ] 1.5 Ensure the input composer remains responsive during heavy streaming

## 2. Validation
- [ ] 2.1 Add snapshot tests for transcript rendering across widths
- [ ] 2.2 Add tests for scrolling invariants (stick-to-bottom vs manual scroll)
- [ ] 2.3 Run formatting, lint, and test suite (cargo fmt, cargo clippy, cargo test)

