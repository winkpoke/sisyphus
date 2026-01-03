# Design: TUI Visual Polish

## Architecture

### 1. Theming System
We will introduce a `Theme` struct in `crates/cli/src/ui/tui/theme.rs` to encapsulate color definitions. This decouples rendering logic from specific color values.

```rust
pub struct Theme {
    pub user: Color,
    pub assistant: Color,
    pub system: Color,
    pub success: Color,
    pub error: Color,
    // ... potentially others like 'border', 'highlight'
}
```

The `Theme` will be instantiated (likely as a default implementation or lazy static) and used throughout the `draw` function in `mod.rs`.

### 2. Toast Notification System
To support ephemeral messages without cluttering the persistent transcript or status bar, we will add a `Toast` state to `TuiState`.

-   **State**:
    ```rust
    pub struct Toast {
        pub message: String,
        pub expires_at: Instant,
        pub kind: ToastKind, // Success, Info, Error
    }
    ```
-   **Lifecycle**:
    -   Triggered by events (e.g., `Ctrl+C` or specific command).
    -   Stored in `TuiState`.
    -   Rendered as an overlay on top of the UI.
    -   Automatically cleared when `Instant::now() > expires_at`.

### 3. Streaming Indicators
To visually indicate that the assistant is still "thinking" or generating text:
-   **Mechanism**: During the rendering loop of transcript items, if an item is flagged as `is_streaming`, we append a visual indicator (cursor block `█` or spinner character) to the displayed text *only* (not the underlying content).
-   **Spinner State**: Re-use the existing `spinner_frame` in `TuiState` to animate the indicator if a spinner is chosen.

## UI/UX Specifics

### Palette
-   **User**: Soft Blue (#5dade2)
-   **Assistant**: Lavender (#a569bd) or Mint (#58d68d)
-   **System**: Muted Grey (#808b96)
-   **Success**: Bright Green (#2ecc71)

### Interactions
-   **Copy**: Pressing `c` on a selected message triggers a "✓ Copied to clipboard" toast for 2 seconds.
