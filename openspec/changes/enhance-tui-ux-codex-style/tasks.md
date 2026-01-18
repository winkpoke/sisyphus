# Tasks: Enhance TUI UX

## Foundation
1. [x] Update `App` layout to remove outer margins (`Margin(0)`).
2. [x] Define new `Theme` colors for ContextBar (`context_bar_bg`, `context_bar_fg`).
3. [x] Add `current_working_directory: String` field to `TuiState` in `state.rs`.
4. [x] Create `MessageBlock` struct wrapping `TranscriptItem` with header styling metadata.

## Components
5. [x] Implement `ContextBar` component (Top Header).
    - [x] Render brand, cwd, and active model in single line.
    - [x] Implement path truncation logic (middle ellipsis for long paths).
    - [x] Handle "Loading..." or "N/A" fallback when model unavailable.
    - [x] Add underline/border pattern for colorblind accessibility.

6. [x] Refactor `Transcript` component to render `MessageBlock`s.
    - [x] Implement styled headers for User/Assistant/System.
    - [x] Add 1 blank line separator between messages (container-managed).
    - [x] Update scroll calculation to include header height (1 line) + separator (1 line).
    - [x] Preserve streaming indicator at end of last line.
    - [x] Add border patterns (solid/dashed) for colorblind accessibility.

7. [x] Refactor `Input` component.
    - [x] Remove "Input" block title and borders.
    - [x] Add `> ` prompt symbol with active/inactive styling.
    - [x] Implement active state (highlight color) and inactive state (system color).
    - [x] Support multi-line input (auto-expand to max 3 lines).
    - [x] Horizontal split: Input area (70%), Status area (30%).

8. [x] Refactor `Status` component to merge with Input.
    - [x] Right-align status in same line as input.
    - [x] Show `[Status] [Tokens]` format (e.g., `[● Connected] [1234/4096]`).
    - [x] Move Session ID from footer to ContextBar (optional, may keep in status).

## Polish
9. [x] Verify scrolling behavior with new MessageBlock layout.
    - [x] Test scroll offset calculation with header heights.
    - [x] Test "stick to bottom" behavior during streaming.
    - [x] Test manual scroll disables stickiness.

10. [x] Add CWD tracking mechanism.
    - [x] Hook into session events for directory changes.
    - [x] Update `current_working_directory` field on directory change.
    - [x] Add tests for CWD update events.

11. [x] Implement terminal resize handling for new layout.
    - [x] Recalculate wrapping on width changes (existing behavior).
    - [x] Adjust ContextBar truncation on resize.
    - [x] Test MessageBlock header adjustment.

12. [x] Implement minimum terminal size detection.
    - [x] Detect terminal < 24×80 columns.
    - [x] Show warning banner in degraded mode.
    - [x] Fallback to prefix-based rendering (no MessageBlock headers).

13. [x] Optimize streaming performance.
    - [x] Re-render only modified message during streaming (not entire transcript).
    - [x] Batch scroll updates (throttle to 60 FPS max).
    - [x] Avoid recalculating layout for every streamed chunk.
    - [x] Add performance benchmarks for streaming scenarios.

## Validation
14. [x] Add unit tests for MessageBlock rendering.
    - [x] Test header rendering for User/Assistant/System.
    - [x] Test scroll calculation with varying message heights.
    - [x] Test streaming indicator placement.

15. [x] Add unit tests for ContextBar.
    - [x] Test path truncation (long paths, short paths).
    - [x] Test model name fallback (Loading..., N/A).
    - [x] Test colorblind accessibility indicators.

16. [x] Add integration tests for full layout.
    - [x] Test minimum terminal size degradation.
    - [x] Test terminal resize event handling.
    - [x] Test streaming performance (measure FPS).

17. [x] Accessibility testing.
    - [x] Verify colorblind differentiation (deuteranopia, protanopia, tritanopia).
    - [x] Test high-contrast terminal compatibility.
    - [x] Verify screen reader compatibility (text structure).

## Documentation
18. [x] Update `openspec/specs/cli-tui/spec.md` with new behaviors.
19. [x] Update README with new TUI features and screenshots.
20. [x] Document degraded mode behavior for small terminals.
