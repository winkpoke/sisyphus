## ADDED Requirements
### Requirement: Clean Interactive Mode Output
The CLI MUST spawn background server subprocesses with stderr piped and consumed silently to prevent log output from interfering with interactive user experience.

#### Scenario: REPL mode suppresses server logs
- **WHEN** user runs `sisyphus repl` without remote URL
- **THEN** CLI spawns a server subprocess with stderr piped
- **AND** server logs are consumed silently in background
- **AND** user sees only clean interactive REPL interface

#### Scenario: TUI mode suppresses server logs
- **WHEN** user runs `sisyphus tui` without remote URL
- **THEN** CLI spawns a server subprocess with stderr piped
- **AND** server logs are consumed silently in background
- **AND** user sees only clean TUI interface

#### Scenario: Server mode shows logs normally
- **WHEN** user runs `sisyphus serve` directly
- **THEN** server subprocess inherits stderr normally
- **AND** all logs are displayed for debugging

#### Scenario: Remote connection unaffected
- **WHEN** user runs interactive mode with `--attach` to remote server
- **THEN** no subprocess is spawned
- **AND** logging behavior is determined by remote server
