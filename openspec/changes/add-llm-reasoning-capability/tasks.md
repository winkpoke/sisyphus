# Tasks

## 1. P0: Safety and protocol correctness

- [x] 1.1 Define reserved-key policy for `request_overrides` and required behavior
- [x] 1.2 Extend event payload to include a reasoning discriminator (`kind`)
- [x] 1.3 Implement override merge with reserved-key protection and tests

## 2. P0: User-visible summary support

- [x] 2.1 Add reasoning config surface (mode/effort/expose/store) with safe defaults
- [x] 2.2 Extend internal request/response models to carry reasoning summary and raw
- [x] 2.3 Map OpenAI-compatible responses into `reasoning_summary` without exposing raw
- [x] 2.4 Emit `kind=reasoning_summary` events only when `expose=summary`
- [x] 2.5 Emit reasoning summary events in core

## 3. P1: Deterministic auto mode

- [x] 3.1 Implement `mode=auto` enablement rules and tests

## 4. P0: TUI integration

- [x] 4.1 Add a local toggle to show/hide `kind=reasoning_summary` transcript entries (default: shown)
- [x] 4.2 Ensure `kind=reasoning_raw` is gated by debug mode and redacted/truncated
- [x] 4.3 Filter reasoning events in TUI transcript rendering

## 5. P0: REPL integration

- [x] 5.1 Add a local toggle to show/hide `kind=reasoning_summary` transcript entries (default: shown)
- [x] 5.2 Ensure `/think` toggles summary visibility locally in the REPL

## 6. P0: Ui command support

- [x] 6.1 Add `/think` UiCommand to toggle summary visibility in interactive UIs
- [x] 6.2 Ensure `/think` is discoverable in the TUI command palette
- [x] 6.3 Register `/think` as a reserved UiCommand name

## 7. Validation

- [x] 7.1 Add tests for merge order, reserved-key rejection, and summary emission
- [x] 7.2 Add tests for `/think` toggle behavior (TUI + REPL)
- [x] 7.3 Run workspace test suite and fix failures
