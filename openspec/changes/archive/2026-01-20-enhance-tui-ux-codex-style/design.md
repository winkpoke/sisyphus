# Design: Codex-style TUI Enhancements

## Context
The current TUI uses a "boxed widgets" layout with margins and heavy borders that reduces content visibility. Modern AI coding assistants (Codex CLI, Claude Code) use immersive, pane-based designs that maximize screen real estate and improve readability.

## Goals / Non-Goals

**Goals:**
- Increase content visibility by 15-20% through margin removal
- Improve message scanning with visual separation
- Provide persistent context awareness (session, directory, model)
- Maintain accessibility for colorblind users
- Preserve streaming performance

**Non-Goals:**
- Customizable layouts or themes (out of scope)
- Mouse interaction support (not planned)
- Multi-session simultaneous viewing (future consideration)

## Visual Hierarchy

The new layout moves away from "boxed widgets" to a "pane-based" terminal UI.

```text
+---------------------------------------------------------------+
|  [ Sisyphus ]  ~/projects/sisyphus             [ gpt-4o ]     | <- Context Header
+---------------------------------------------------------------+
|                                                               |
|  [ User ] --------------------------------------------------  | <- Message Block Header
|  Show me how to implement the context header.                 |
|                                                               |
|  [ Assistant ] ---------------------------------------------  |
|  Here is the implementation plan...                           |
|                                                               |
|  ... (scrolling content) ...                                  |
|                                                               |
+---------------------------------------------------------------+
|  > |                                            [ Connected ] | <- Input/Status Footer
+---------------------------------------------------------------+
```

## Components

### 1. Immersive Layout
- **Constraint**: `Margin(0)` on the root layout.
- **Borders**: Remove outer borders. Use dividers or subtle background changes to separate sections.

### 2. Context Header
- **Position**: Top line (Height: 1).
- **Style**: Distinct background color (theme: `context_bar_bg`, `context_bar_fg`).
- **Content**:
    - **Left**: Brand (`[ Sisyphus ]`)
    - **Center**: Current Working Directory (truncated with ellipsis in middle if too long: `/Users/.../sisyphus`)
    - **Right**: Active Model name (fallback: "Loading..." or "N/A" if unavailable)
- **Accessibility**: Add underline or distinct border pattern for colorblind differentiation
- **State tracking**: Add `current_working_directory: String` field to `TuiState` and update when directory changes via session events

### 3. Message Blocks (Transcript)
- **Concept**: Treat messages as distinct "cards" in the stream with per-message headers.
- **Header**:
    - User: Blue accent (existing theme: `user: Color::Rgb(93, 173, 226)`).
    - Assistant: Lavender accent (existing theme: `assistant: Color::Rgb(165, 105, 189)`).
    - System: Grey/Dim (existing theme: `system: Color::Rgb(128, 139, 150)`).
    - Header text format: `[ User ]`, `[ Assistant ]`, `[ System ]` with right-aligned dashes
    - Accessibility: Add border pattern (solid vs dashed) in addition to color for colorblind users
- **Body**: Standard text rendering below header, no borders.
- **Streaming indicator**: Spinner character displayed at end of last line (existing behavior preserved).
- **Separators**: 1 blank line between messages (part of container layout, not MessageBlock).
- **Scroll calculation**: Must include header height (1 line) + blank separator (1 line) per message when calculating scroll offsets.
- **State**: Introduce `MessageBlock` struct wrapping `TranscriptItem` with header styling metadata.

### 4. Unified Footer (Input + Status)
- **Position**: Bottom, single line (Height: 1-3 depending on input length).
- **Layout**: Horizontal split - Left: Input area (70%), Right: Status area (30%).
- **Input**:
    - Remove "Input" titled block.
    - Use `> ` prompt symbol (theme: `highlight` color).
    - Active state: Prompt character highlighted in `highlight` color
    - Inactive state: Prompt character in `system` color
    - Multi-line input: Auto-expand to fit content (max 3 lines)
- **Status**:
    - Right-aligned in same line as input
    - Show: `[Status] [Tokens]` (e.g., `[● Connected] [1234/4096]`)
    - Use existing theme colors (`success`, `error`, `highlight`)

### 5. Constraints & Performance
- **Minimum terminal size**: 24 rows × 80 columns.
    - On smaller terminals: Show warning banner, render in degraded mode (no MessageBlock headers, revert to prefixes).
- **Streaming performance**:
    - Re-render only modified message during streaming (not entire transcript).
    - Batch scroll updates (throttle to 60 FPS max).
    - Avoid recalculating layout for every streamed chunk.
- **Resize handling**:
    - Recalculate wrapping on terminal width changes (existing behavior preserved).
    - Adjust message block headers to new width.

## Decisions

### Decision 1: Single-line footer (not Powerline style)
**What**: Status indicators integrated into input line on the right, not a separate line below.

**Why**: Maximizes vertical space for transcript, simpler implementation, matches existing pattern.

**Alternatives**:
- **Powerline style (2 lines)**: More visual separation but reduces transcript area by 1 line (~4% on 24-line terminal).
- **Popup status**: Disrupts reading flow during status changes.

### Decision 2: Keep existing theme colors
**What**: Use current `theme.user` (Blue), `theme.assistant` (Lavender), `theme.system` (Grey) instead of new Green/Cyan.

**Why**: Maintains consistency with existing design, reduces migration risk, colors already tested for contrast.

**Alternatives**:
- **New palette (Green/Cyan)**: Would require theme migration and user adjustment.

### Decision 3: Separator lines in container, not MessageBlock
**What**: Blank lines between messages managed by transcript container, not per-message state.

**Why**: Simpler state, easier to scroll calculation, consistent with "stick to bottom" behavior.

**Alternatives**:
- **Separator in MessageBlock**: Adds complexity to per-message state, harder to manage dynamically.

## Risks / Trade-offs

### Risk 1: Scrolling performance degradation with MessageBlock layout
**Mitigation**: Optimize scroll calculation to pre-calculate heights, batch updates during streaming.

### Risk 2: Colorblind accessibility issues
**Mitigation**: Add border patterns (solid vs dashed) and brightness indicators alongside color differentiation.

### Risk 3: Small terminal incompatibility
**Mitigation**: Graceful degradation mode with prefix-based fallback when terminal < 24×80.

### Trade-off: Minimal header vs informative header
**Choice**: Show only 3 critical fields (brand, cwd, model) in header to minimize vertical space.
**Impact**: Session ID moved to status footer, tokens shown in footer (reduces header clutter).
