# Implementation Tasks

1.  [x] Create `crates/cli/src/ui/repl.rs` and extract `reedline` loop logic.
    - Validation: `cargo check -p sisyphus`
2.  [x] Create `crates/cli/src/bootstrap.rs` and extract `Agent` initialization logic.
    - Validation: `cargo check -p sisyphus`
3.  [x] Create `crates/cli/src/commands/` module and move `run_chat`/`run_serve` logic.
    - Validation: `cargo check -p sisyphus`
4.  [x] Refactor `crates/cli/src/main.rs` to use new modules.
    - Validation: `cargo build -p sisyphus`
    - Validation: Manual test `sisyphus chat` and `sisyphus serve`
