## ADDED Requirements

### Requirement: Permission Configuration
The system MUST support configuring permissions per tool via configuration, without requiring code changes for standard tools, and it MUST preserve safe defaults for tools that are not explicitly configured.

#### Scenario: Overriding tool permission
- **GIVEN** an agent configuration
- **WHEN** I set `permissions.overrides["my_tool"] = "Deny"`
- **THEN** `my_tool` is denied execution
- **AND** other tools follow their default permission behavior

#### Scenario: Unspecified tools remain safe
- **GIVEN** an agent configuration with no override for `new_tool`
- **WHEN** `new_tool` is requested for execution
- **THEN** `new_tool` is evaluated under a safe default policy (Ask/Deny)
