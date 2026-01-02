# Change: Add Codex-like polish features to the Sisyphus TUI

## Why
Codex-like polish is defined by more than a basic transcript: it includes discoverability (command palette), safe overlays for approvals/errors/help, transcript selection/copy, and small interaction details (key hints, notifications) that make long sessions comfortable.

## What Changes
- Add a command palette that surfaces slash commands and supports keyboard navigation.
- Add overlays/popups for help, errors, and long content viewing.
- Add transcript selection and copy (to clipboard where supported).
- Add key-hint affordances and unobtrusive status indicators.

## Impact
- Affected specs: cli-tui (modified), slash-commands (modified)
- Affected code (expected): TUI widgets/state; slash command registry exposure; platform clipboard integration

## Non-Goals
- Full parity with Codex’s onboarding flows.
- Implementing sandbox/approval execution policies unless required by agent/tooling.

