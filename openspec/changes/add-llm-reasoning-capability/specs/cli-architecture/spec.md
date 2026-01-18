## ADDED Requirements

### Requirement: REPL supports /think UiCommand for reasoning summaries
The CLI REPL SHALL support `/think` as a UiCommand that toggles local visibility of reasoning summaries.

#### Scenario: Summaries are visible by default
- **GIVEN** the CLI is running in REPL mode
- **AND** the user has not executed `/think` to disable summaries
- **WHEN** the REPL receives a `MessageReceived` system event with `kind = "reasoning_summary"`
- **THEN** the REPL MUST render it as a system transcript entry

#### Scenario: /think hides reasoning summaries
- **GIVEN** the CLI is running in REPL mode
- **WHEN** the user executes `/think` to disable reasoning summary visibility
- **AND** the REPL receives a `MessageReceived` system event with `kind = "reasoning_summary"`
- **THEN** the REPL MUST NOT render that transcript entry

#### Scenario: /think is local and does not reach the server
- **GIVEN** the CLI is running in REPL mode
- **WHEN** the user inputs `/think`
- **THEN** the CLI MUST execute the UiCommand locally
- **AND** it MUST NOT send `/think` to the chat endpoint

