# Change: Refactor TUI Architecture

## Why
The current TUI implementation suffers from architectural issues:
- **Monolithic `run` loop**: Handles event polling, logic, and rendering in one place.
- **Tight Coupling**: Rendering logic is mixed with business logic.
- **Hard-to-test state mutations**: State changes are ad-hoc and scattered.
Refactoring to the Model-View-Update (MVU) pattern will improve maintainability, testability, and scalability.

## What Changes
- Refactor `crates/cli/src/ui/tui` modules.
- Introduce `Action` enum for all state mutations.
- Separate `Update` logic into a pure function (or near-pure).
- Extract UI rendering into reusable components.
- Ensure no functional regression (behavior remains the same).

## Impact
- **Specs**: `cli-tui`
- **Code**: `crates/cli/src/ui/tui`
