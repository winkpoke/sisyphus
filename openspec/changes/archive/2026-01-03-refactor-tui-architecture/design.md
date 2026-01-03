# TUI Architecture Design

## Pattern: Model-View-Update (MVU)

We will adopt an MVU-inspired architecture, commonly used with `ratatui` apps.

### Components

1.  **Model (`App` struct)**
    *   Holds the entire application state (`TuiState`).
    *   Source of truth.
    *   Defined in `app.rs`.

2.  **Message (`Action` enum)**
    *   Represents all possible events that can change the state.
    *   Examples: `Tick`, `KeyPress(key)`, `ServerMessage(msg)`, `Resize(w, h)`.
    *   Defined in `action.rs`.

3.  **Update (`update` function)**
    *   Signature: `fn update(app: &mut App, action: Action) -> Command`
    *   Handles business logic.
    *   Mutates `App` state based on `Action`.
    *   Returns a `Command` (optional) for side effects (e.g., "send network request", "quit").
    *   Defined in `update.rs`.

4.  **View (`ui::render` function)**
    *   Signature: `fn render(app: &App, frame: &mut Frame)`
    *   Pure function that draws the UI based on `App` state.
    *   Defined in `ui/mod.rs` and submodules.

### Directory Structure

```text
crates/cli/src/ui/tui/
├── mod.rs           // Public API, sets up the runtime loop
├── app.rs           // App struct and initialization
├── action.rs        // Action enum
├── update.rs        // State transition logic
├── event.rs         // Event polling (crossterm -> Action)
└── ui/              // Presentation Layer
    ├── mod.rs       // Main layout
    ├── components/  // Reusable widgets
    │   ├── transcript.rs
    │   ├── input.rs
    │   ├── overlay.rs
    │   └── help.rs
    └── theme.rs     // Colors and styles
```

### Data Flow

1.  `mod.rs` runs the main loop.
2.  `event.rs` captures keyboard/mouse/network events and converts them to `Action`.
3.  `update.rs` applies `Action` to `App` state.
4.  `ratatui` draws the UI using `ui::render(&App)`.

### Benefits
- **Testability**: `update` function can be unit tested without a terminal.
- **Separation of Concerns**: Rendering code doesn't know about business logic.
- **Predictability**: State changes are centralized.
