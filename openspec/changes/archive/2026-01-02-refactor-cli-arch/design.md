# CLI Architecture Design

## Current Architecture Analysis
- `main.rs` is a "God Class" handling args, config, logging, server, and chat loop.
- `run_serve` tightly couples agent assembly.
- `run_chat` and `run_attach` duplicate REPL logic.

## Proposed Architecture

### 1. Modular File Structure
```text
crates/cli/src/
├── commands/           # Command implementations
│   ├── mod.rs
│   ├── chat.rs         # Chat & Attach logic (unified)
│   └── serve.rs        # Server startup logic
├── ui/                 # UI & Interaction
│   ├── mod.rs
│   ├── banner.rs       # Existing banner
│   ├── repl.rs         # The Read-Eval-Print Loop abstraction
│   └── completer.rs    # Existing completer
├── bootstrap.rs        # "Composition Root" - Logic to wire up the Agent
├── server_manager.rs   # Existing process manager
└── main.rs             # Minimal entry point (Args parsing only)
```

### 2. Separation of Concerns
- **`main.rs`**: Only parses `clap` arguments and dispatches to `commands::*`.
- **`bootstrap.rs`**: Factory function `build_agent(config)` to initialize `Agent`, `EventBus`, and `Tools`.
- **`ui/repl.rs`**: Reusable `Repl` struct encapsulating `reedline` and input loop.

### 3. Migration Strategy
1.  **Extract REPL**: Move `reedline` loop to `Repl` class.
2.  **Extract Bootstrap**: Move agent setup to `bootstrap.rs`.
3.  **Split Commands**: Move logic to `commands/`.
4.  **Simplify Main**: Reduce `main.rs` to dispatcher.
