# agent-core Specification

## Purpose
Defines the core architecture for Sisyphus Agents, including configuration, permissions, and dynamic prompt generation. This specification ensures agents are secure, configurable, and context-aware, matching OpenCode's capabilities.

## Requirements
### Requirement: Agent Configuration
The system SHALL support configuring Agents with strictly typed metadata including name, mode, permissions, and model settings.

#### Scenario: Load Valid Config
- **WHEN** a valid agent configuration JSON/TOML is loaded
- **THEN** the system correctly parses the name, mode, and permission rules

### Requirement: Agent Permissions
The system SHALL enforce granular permissions for file editing, shell execution, and skills, supporting Allow, Ask, and Deny levels.

#### Scenario: Default Permissions
- **WHEN** no permissions are specified
- **THEN** the system applies safe default permissions (e.g., Edit=Allow, but risky shell commands=Ask/Deny)

### Requirement: Dynamic System Prompt
The system SHALL generate system prompts dynamically at the start of a chat session, including current environment context and custom project rules.

#### Scenario: Environment Injection
- **WHEN** a system prompt is generated
- **THEN** it includes the current Working Directory, Platform (OS), and Today's Date in a formatted `<env>` block

#### Scenario: Custom Rules Injection
- **WHEN** `AGENTS.md` exists in the workspace
- **THEN** its content is appended to the system prompt

