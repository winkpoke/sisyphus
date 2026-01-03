# Tasks

1. [x] Enforce tool-execution permissions at the tool boundary
   - Validation: add tests for Deny and Ask behaviors, including exact message content and no tool execution

2. [x] Snapshot and assemble deterministic system prompt per user turn
   - Validation: add tests that the `<env>` labels match spec and prompt content is stable across tool-call loops
   - Validation: add tests that `AGENTS.md` content is appended verbatim when present and is snapshotted per turn

3. [x] Generate `/help` output from the command registry with deterministic ordering
   - Validation: add tests for the exact header and sorted command listing

4. [x] Improve slash command argument parsing for quoted strings
   - Validation: add tests for quoted multi-word arguments, escapes inside quotes, and unterminated-quote error message

5. [x] Apply clear-history and new-session command effects to session state consistently
   - Validation: add tests that the next completion request includes no prior messages or tool results

6. [x] Run workspace test suite and fix failures
