## ADDED Requirements

### Requirement: Command Parsing
The command parsing logic MUST be encapsulated in a dedicated component, separating syntax analysis from command execution, and it MUST expose enough information for reliable custom command expansion.

#### Scenario: Parsing quoted arguments
- **GIVEN** a command string `/cmd "arg 1" arg2`
- **WHEN** parsed by the command parser
- **THEN** it returns command `/cmd` and arguments `["arg 1", "arg2"]`

#### Scenario: Preserving raw args for expansion
- **GIVEN** a command string `/cmd "arg 1" arg2`
- **WHEN** parsed by the command parser
- **THEN** it also returns `raw_args` equal to `"arg 1" arg2`
