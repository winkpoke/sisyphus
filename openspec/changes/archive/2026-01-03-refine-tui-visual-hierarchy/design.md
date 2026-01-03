# Design: Refine TUI Visual Hierarchy

## State Management
We will extend `TuiState` to track:
- `AppStatus`: Connection state (Connected, Disconnected, Processing).
- `spinner_frame`: For animation.
- `active_model`: Name of the current LLM.
- `token_usage`: Usage stats string.
- `context_title`: Dynamic title for the transcript.

## Layout Strategy
### Status Bar
The bottom area (previously `Constraint::Length(1)`) will be split horizontally:
- **Left (25%)**: Session ID / Context.
- **Center (50%)**: Active Model & Token Usage.
- **Right (25%)**: System Status (Spinner/Connection Icon).

### Transcript
- Apply `Padding` to the `Block` to prevent text from touching borders.
- Replace static "Transcript" title with dynamic `context_title`.

## Visual Style
- **Status Indicators**: Green for Connected, Red for Disconnected, Spinner for Processing.
- **Headers**: Bold Cyan for visibility.
- **Padding**: 2 chars horizontal, 1 char vertical.
