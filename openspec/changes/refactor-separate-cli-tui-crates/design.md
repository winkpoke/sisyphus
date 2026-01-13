# Design: Separate CLI and TUI into Distinct Crates

## Context

The current `crates/cli` crate serves dual purposes:
1. **Headless CLI**: Server management, REPL interface, command routing
2. **TUI Client**: Full terminal UI with MVU architecture, rendering, and interactive components

This coupling causes:
- Dependency bloat: REPL pulls in ratatui, crossterm, arboard unnecessarily
- Architectural violation: Project aims to be "headless by design" per README
- Testing complexity: Cannot test UI modes independently
- Build inefficiency: All users compile TUI code even if they only need CLI

## Goals / Non-Goals

### Goals
- Separate TUI into its own crate with isolated dependencies
- Enable CLI-only builds without TUI dependencies
- Maintain backward compatibility for TUI via feature flag
- Keep shared utilities (banner, completer) accessible to both
- Support future UI clients (GUI, web) without CLI bloat

### Non-Goals
- Changing the TUI implementation or MVU architecture
- Modifying the REPL implementation or behavior
- Creating a new shared UI abstraction layer
- Changing the external command-line interface or behavior

## Decisions

### Decision 1: TUI as Separate Crate in Workspace
**Rationale**: Full separation of concerns, independent versioning, clear dependency boundaries.

**Alternatives considered**:
- Keep in same crate with features (rejected: still builds TUI code)
- Move to external repository (rejected: tight coupling with core)
- Create shared UI crate (rejected: YAGNI, premature abstraction)

### Decision 2: TUI as Optional Feature in CLI
**Rationale**: Backward compatibility, simple opt-in mechanism.

**Implementation**:
```toml
[features]
default = []
tui = ["dep:tui"]
```

### Decision 3: Shared Utilities Remain in CLI
**Rationale**: Banner and completer are CLI-focused utilities used by both modes.

**Alternatives considered**:
- Move to common crate (rejected: too generic, CLI-specific)
- Duplicate in both crates (rejected: maintenance burden)
- Create shared-ui crate (rejected: YAGNI, only 2 files)

### Decision 4: TUI Crate Depends on CLI
**Rationale**: TUI needs CLI's client management, session handling, server manager.

**Implementation**:
```toml
# crates/tui/Cargo.toml
[dependencies]
cli = { path = "../cli" }
```

## Risks / Trade-offs

### Risk 1: Circular Dependency
**Mitigation**: TUI depends on CLI, CLI only depends on TUI via optional feature. Verified with `cargo check`.

### Risk 2: Build Complexity
**Mitigation**: Clear feature flags, documented in Cargo.toml and README. Default builds exclude TUI.

### Risk 3: Code Movement Breakage
**Mitigation**: Incremental moves with `mod` aliases, update imports systematically, test after each move.

### Trade-off: More Crates to Manage
**Benefit outweighs cost**: 2 crates is still simple, clearer architecture pays off long-term.

## Migration Plan

### Phase 1: Prepare (No Breaking Changes)
1. Create `crates/tui` skeleton with Cargo.toml
2. Add `tui` feature to `crates/cli/Cargo.toml`
3. Update workspace `Cargo.toml` members list

### Phase 2: Move TUI Code
1. Move `crates/cli/src/ui/tui/` → `crates/tui/src/`
2. Update internal imports (remove `crate::` references)
3. Create `mod tui` alias in CLI for backward compatibility

### Phase 3: Update Dependencies
1. Remove TUI deps from `crates/cli/Cargo.toml` when feature not enabled
2. Add CLI dependency to `crates/tui/Cargo.toml`
3. Update `crates/cli/src/commands/chat.rs` to use optional TUI

### Phase 4: Validate
1. Build CLI-only: `cargo build --bin sisyphus`
2. Build with TUI: `cargo build --bin sisyphus --features tui`
3. Run tests: `cargo test`
4. Manual test: `cargo run --release --bin sisyphus -- chat --tui`

### Rollback
If issues arise:
1. Revert workspace Cargo.toml
2. Move code back to `crates/cli/src/ui/tui/`
3. Restore dependencies
4. Git checkout to pre-change state

## Open Questions

None. Scope is well-defined based on current code structure.

## File Structure After Migration

```
crates/
├── cli/
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── commands/             # Command routing
│   │   ├── ui/
│   │   │   ├── repl.rs           # REPL implementation
│   │   │   ├── banner.rs         # Shared: startup banner
│   │   │   ├── completer.rs      # Shared: command completion
│   │   │   └── mod.rs
│   │   ├── server_manager.rs
│   │   └── bootstrap.rs
│   └── Cargo.toml                # TUI as optional feature
└── tui/
    ├── src/
    │   ├── mod.rs                # Public interface
    │   ├── app.rs                # TUI app struct
    │   ├── state.rs
    │   ├── update.rs
    │   ├── action.rs
    │   ├── event.rs
    │   ├── terminal.rs
    │   ├── theme.rs
    │   ├── transcript.rs
    │   ├── commands.rs
    │   └── ui/                   # UI components
    │       ├── mod.rs
    │       ├── components/
    │       └── utils.rs
    └── Cargo.toml                # TUI-specific deps
```
