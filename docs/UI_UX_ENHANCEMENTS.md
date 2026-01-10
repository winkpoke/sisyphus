# UI/UX Enhancement Proposal for Sisyphus TUI

## 1. Executive Summary
This document outlines proposed enhancements to the Sisyphus CLI TUI to align with the PRD requirements for a "rich interactive TUI" and improve the overall user experience. The focus is on visual hierarchy, feedback, and usability.

## 2. Visual Hierarchy & Layout

### 2.1 Enhanced Transcript Rendering
**Current State**: Plain text rendering with basic coloring.
**Proposal**:
- **Markdown Support**: Render messages using Markdown (headers, lists, bold/italic).
- **Code Highlighting**: Syntax highlighting for code blocks within the transcript.
- **Visual Separation**: Clearer distinction between User and Assistant messages using background colors or distinct borders.

### 2.2 Dynamic Status Bar
**Current State**: Basic status text.
**Proposal**:
- **Mode Indicators**: Visual indicators for current mode (Insert, Normal, Selection).
- **Spinner**: Animated spinner for background activities.
- **Context Info**: Display current working directory, active model, and token usage prominently.

### 2.3 Key Hints (Footer)
**Current State**: None.
**Proposal**:
- **Context-Aware Hints**: Show available keybindings based on the current mode (e.g., `Esc` to exit input, `Ctrl+s` to send).

## 3. Interactive Elements

### 3.1 Input Editor
**Current State**: Basic single-line input.
**Proposal**:
- **Multi-line Editing**: Support for multi-line input (essential for coding prompts).
- **Cursor Navigation**: Full cursor movement (Home, End, word jumps).
- **History**: Command history navigation (Up/Down).

### 3.2 Visual Feedback (Toasts)
**Current State**: None.
**Proposal**:
- **Toast Notifications**: Transient overlays for actions like "Copied to clipboard", "Model switched", or errors.
- **Positioning**: Top-right or bottom-right floating notifications.

### 3.3 Command Palette
**Current State**: Basic implementation exists.
**Proposal**:
- **Fuzzy Search**: Improved filtering of commands.
- **Categories**: Group commands by category (e.g., Session, Agent, System).

## 4. Theming

### 4.1 Semantic Colors
**Current State**: Basic theme struct.
**Proposal**:
- **Expanded Palette**: Define specific colors for UI elements (borders, headers, code backgrounds).
- **Configurability**: Allow users to override theme colors via `sisyphus.toml`.

## 5. Implementation Plan

### Phase 1: Quick Wins (Immediate)
- [ ] Implement **Key Hints** footer.
- [ ] Implement **Toast Notification** system.
- [ ] Improve **Status Bar** layout.

### Phase 2: Core Interaction
- [ ] Upgrade **Input Editor** to support multi-line and navigation.
- [ ] Enhance **Command Palette** UI.

### Phase 3: Advanced Rendering
- [ ] Integrate Markdown rendering for **Transcript**.
- [ ] Add Syntax Highlighting.
