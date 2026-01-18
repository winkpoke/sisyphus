# Tasks: Enhance TUI UX

## Foundation
1. [ ] Update `App` layout to remove outer margins (`Margin(0)`).
2. [ ] Define new `Theme` colors for ContextBar (`context_bar_bg`, `context_bar_fg`).
3. [ ] Add `current_working_directory: String` field to `TuiState` in `state.rs`.
4. [ ] Create `MessageBlock` struct wrapping `TranscriptItem` with header styling metadata.

## Components
5. [ ] Implement `ContextBar` component (Top Header).
    - [ ] Render brand, cwd, and active model in single line.
    - [ ] Implement path truncation logic (middle ellipsis for long paths).
    - [ ] Handle "Loading..." or "N/A" fallback when model unavailable.
    - [ ] Add underline/border pattern for colorblind accessibility.

6. [ ] Refactor `Transcript` component to render `MessageBlock`s.
    - [ ] Implement styled headers for User/Assistant/System.
    - [ ] Add 1 blank line separator between messages (container-managed).
    - [ ] Update scroll calculation to include header height (1 line) + separator (1 line).
    - [ ] Preserve streaming indicator at end of last line.
    - [ ] Add border patterns (solid/dashed) for colorblind accessibility.

7. [ ] Refactor `Input` component.
    - [ ] Remove "Input" block title and borders.
    - [ ] Add `> ` prompt symbol with active/inactive styling.
    - [ ] Implement active state (highlight color) and inactive state (system color).
    - [ ] Support multi-line input (auto-expand to max 3 lines).
    - [ ] Horizontal split: Input area (70%), Status area (30%).

8. [ ] Refactor `Status` component to merge with Input.
    - [ ] Right-align status in same line as input.
    - [ ] Show `[Status] [Tokens]` format (e.g., `[● Connected] [1234/4096]`).
    - [ ] Move Session ID from footer to ContextBar (optional, may keep in status).

## Polish
9. [ ] Verify scrolling behavior with new MessageBlock layout.
    - [ ] Test scroll offset calculation with header heights.
    - [ ] Test "stick to bottom" behavior during streaming.
    - [ ] Test manual scroll disables stickiness.

10. [ ] Add CWD tracking mechanism.
    - [ ] Hook into session events for directory changes.
    - [ ] Update `current_working_directory` field on directory change.
    - [ ] Add tests for CWD update events.

11. [ ] Implement terminal resize handling for new layout.
    - [ ] Recalculate wrapping on width changes (existing behavior).
    - [ ] Adjust ContextBar truncation on resize.
    - [ ] Test MessageBlock header adjustment.

12. [ ] Implement minimum terminal size detection.
    - [ ] Detect terminal < 24×80 columns.
    - [ ] Show warning banner in degraded mode.
    - [ ] Fallback to prefix-based rendering (no MessageBlock headers).

13. [ ] Optimize streaming performance.
    - [ ] Re-render only modified message during streaming (not entire transcript).
    - [ ] Batch scroll updates (throttle to 60 FPS max).
    - [ ] Avoid recalculating layout for every streamed chunk.
    - [ ] Add performance benchmarks for streaming scenarios.

## Validation
14. [ ] Add unit tests for MessageBlock rendering.
    - [ ] Test header rendering for User/Assistant/System.
    - [ ] Test scroll calculation with varying message heights.
    - [ ] Test streaming indicator placement.

15. [ ] Add unit tests for ContextBar.
    - [ ] Test path truncation (long paths, short paths).
    - [ ] Test model name fallback (Loading..., N/A).
    - [ ] Test colorblind accessibility indicators.

16. [ ] Add integration tests for full layout.
    - [ ] Test minimum terminal size degradation.
    - [ ] Test terminal resize event handling.
    - [ ] Test streaming performance (measure FPS).

17. [ ] Accessibility testing.
    - [ ] Verify colorblind differentiation (deuteranopia, protanopia, tritanopia).
    - [ ] Test high-contrast terminal compatibility.
    - [ ] Verify screen reader compatibility (text structure).

## Documentation
18. [ ] Update `openspec/specs/cli-tui/spec.md` with new behaviors.
19. [ ] Update README with new TUI features and screenshots.
20. [ ] Document degraded mode behavior for small terminals.
