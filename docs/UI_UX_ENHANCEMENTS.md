# UI/UX Enhancement Proposal for Sisyphus TUI

## 1. Executive Summary
This document outlines proposed enhancements to the Sisyphus CLI TUI to align with the PRD requirements for a "rich interactive TUI" and improve the overall user experience. The focus is on visual hierarchy, feedback, and usability.

## 2. Visual Hierarchy & Layout

### 2.1 Enhanced Transcript Rendering
**Current State**: Plain text rendering with basic coloring.
**Status**: ✅ **PARTIALLY COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Message Blocks**: Distinct visual blocks with styled headers for User/Assistant/System messages
- **Visual Separation**: Kind-specific headers with semantic colors (Blue for User, Lavender for Assistant, Grey for System)
- **Accessibility**: Colorblind-friendly design with structural cues (border patterns: solid/dashed/dotted)
- **Separators**: 1-blank-line separator between messages
- **Streaming Indicator**: Blinking cursor/spinner at end of incomplete messages
- **Remaining**:
  - Markdown Support (headers, lists, bold/italic)
  - Code Highlighting for code blocks

### 2.2 Dynamic Status Bar
**Current State**: Basic status text.
**Status**: ✅ **COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Context Header**: Persistent header bar at top with application brand, working directory, and active model
- **Unified Footer**: Combined input and status area (70% input, 30% status) at bottom
- **Status Format**: `[Status] [Tokens]` format with connection status indicators
- **Mode Indicators**: Visual indicators for current mode (active/inactive input state)
- **Spinner**: Animated spinner during streaming responses
- **Context Info**: Working directory displayed in Context Header, token usage in Status Footer

### 2.3 Key Hints (Footer)
**Current State**: None.
**Status**: ✅ **COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Context-Aware Hints**: Available keybindings documented in command palette
- **New content indicator**: Visual indicator when new content arrives while scrolled up
- **Status indicators**: Connection status, token usage displayed in unified footer

## 3. Interactive Elements

### 3.1 Input Editor
**Current State**: Basic single-line input.
**Status**: ✅ **PARTIALLY COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Minimalist Design**: Prompt-driven input with `> ` symbol, no heavy borders
- **Focus Indication**: Active state (highlight color) vs inactive state (system color)
- **Multi-line Editing**: Support for multi-line input with auto-expansion (max 3 lines)
- **Remaining**:
  - Full cursor navigation (Home, End, word jumps)
  - Command history navigation (Up/Down)

### 3.2 Visual Feedback (Toasts)
**Current State**: None.
**Status**: ✅ **COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Toast Notifications**: Transient overlays for actions (e.g., "✓ Copied")
- **Positioning**: On-screen notification that auto-disappears after 2 seconds
- **Copy Feedback**: Toast notification when text is copied to clipboard

### 3.3 Command Palette
**Current State**: Basic implementation exists.
**Proposal**:
- **Fuzzy Search**: Improved filtering of commands.
- **Categories**: Group commands by category (e.g., Session, Agent, System).

## 4. Theming

### 4.1 Semantic Colors
**Current State**: Basic theme struct.
**Status**: ✅ **PARTIALLY COMPLETED** (archived: `2026-01-20-enhance-tui-ux-codex-style`)
**Implementation**:
- **Expanded Palette**: Semantic colors defined for message types (user: Blue, assistant: Lavender, system: Grey)
- **Theme Colors**: Context bar (background, foreground), status indicators (success, error, highlight)
- **Colorblind Support**: Structural cues (border patterns, separators) alongside color differentiation
- **Remaining**:
  - User configurability via `sisyphus.toml`

## 5. Implementation Plan

### Phase 1: Quick Wins (Immediate)
- [x] Implement **Key Hints** footer. (✅ Completed)
- [x] Implement **Toast Notification** system. (✅ Completed)
- [x] Improve **Status Bar** layout. (✅ Completed - Context Header + Unified Footer)

### Phase 2: Core Interaction
- [x] Upgrade **Input Editor** to support multi-line. (✅ Completed - Max 3 lines)
- [ ] Add full cursor navigation (Home, End, word jumps).
- [ ] Add command history navigation (Up/Down).
- [ ] Enhance **Command Palette** UI.

### Phase 3: Advanced Rendering
- [ ] Integrate Markdown rendering for **Transcript**.
- [ ] Add Syntax Highlighting for code blocks.
- [ ] Make theme colors configurable via `sisyphus.toml`.
