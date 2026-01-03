# Tasks

## 1. P0: Safety and protocol correctness
- [ ] 1.1 Define reserved-key policy for `request_overrides` and required behavior
- [ ] 1.2 Extend event payload to include a reasoning discriminator (`kind`)
- [ ] 1.3 Implement override merge with reserved-key protection and tests

## 2. P0: User-visible summary support
- [ ] 2.1 Add reasoning config surface (mode/effort/expose/store) with safe defaults
- [ ] 2.2 Extend internal request/response models to carry reasoning summary and raw
- [ ] 2.3 Map OpenAI-compatible responses into `reasoning_summary` without exposing raw
- [ ] 2.4 Emit `kind=reasoning_summary` events only when `expose=summary`

## 3. P1: Deterministic auto mode
- [ ] 3.1 Implement `mode=auto` enablement rules and tests

## 4. P1: TUI integration
- [ ] 4.1 Add a local toggle to show/hide `kind=reasoning_summary` transcript entries
- [ ] 4.2 Ensure `kind=reasoning_raw` is gated by debug mode and redacted/truncated

## 5. P1: Slash command support
- [ ] 5.1 Add `/think` command to toggle session summary output
- [ ] 5.2 Ensure `/think` is discoverable in the command palette

## 6. Validation
- [ ] 6.1 Add tests for merge order, reserved-key rejection, and summary emission
- [ ] 6.2 Add tests for `/think` toggle behavior
- [ ] 6.3 Run workspace test suite and fix failures
