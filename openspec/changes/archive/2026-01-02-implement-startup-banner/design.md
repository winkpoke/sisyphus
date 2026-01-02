# Design: Startup Banner

## User Interface

The startup screen will follow this layout:

```text
 <ORANGE_ASCII_LOGO>

╭─ >_ Sisyphus (v0.1.0) ───────────────────────────────────╮
│                                                          │
│  model:     gpt-4-turbo         /model to change         │
│  directory: D:\Projects\2025\sisyphus                    │
│                                                          │
╰──────────────────────────────────────────────────────────╯

Tip: If a turn went sideways, /undo asks Sisyphus to revert.
```

### Colors
- **Logo**: Orange (ANSI 208 or RGB 227, 87, 40).
- **Box Borders**: White/Light Grey.
- **Header Text**: White/Bold.
- **Labels**: Grey/Dimmed.
- **Values**: White/Bright.
- **Commands/Hints**: Blue/Cyan (e.g., `/model`, `/undo`).

## Technical Architecture

### Module: `crates/cli/src/banner.rs`
A dedicated module will handle the rendering logic to keep `main.rs` clean.

### Dependencies
- `colored`: For ANSI color codes and styling.

### Data Sources
- **Version**: `env!("CARGO_PKG_VERSION")` (compile-time).
- **Directory**: `std::env::current_dir()` (runtime).
- **Configuration**: `common::config::Config` (runtime). The `model` field MUST be retrieved from the loaded configuration struct, ensuring it reflects any overrides (e.g., from environment variables or config files). Hardcoded defaults in the UI are strictly prohibited.

### Implementation Logic
1.  **Logo Rendering**: Print the static ASCII string with the orange color applied.
2.  **Data Retrieval**:
    -   Load `Config` using `Config::load()`.
    -   Get current directory.
    -   Get package version.
3.  **Box Construction**:
    -   Calculate the width of the box (fixed or dynamic).
    -   Use Unicode box-drawing characters (`╭`, `─`, `╮`, `│`, `╰`, `╯`).
    -   Format the inner content with padding to align columns.
3.  **Tip Display**: Print a tip string below the box.
