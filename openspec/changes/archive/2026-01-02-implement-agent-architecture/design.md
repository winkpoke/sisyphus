## Context
We are porting the core Agent architecture from OpenCode (TypeScript) to Sisyphus (Rust). OpenCode's architecture heavily relies on a separation between the static Agent definition (Config) and the runtime Agent instance, along with dynamic system prompt generation that includes environment details and custom project rules.

## Goals
- **Separation of Concerns**: Strictly separate Agent configuration (metadata, permissions) from runtime state (session, tools).
- **Security**: Implement granular permission controls (Edit, Bash, Skill) mirroring OpenCode's `Permission` schema.
- **Context Awareness**: Enable dynamic system prompts that inject current environment context (CWD, OS, Date) and custom user instructions (`AGENTS.md`).
- **Maintainability**: Use strong typing for configurations and permissions.
- **Future Compatibility**: Design structures to support sub-agents and multi-agent orchestration (e.g., via `AgentMode`) to align with OpenCode.

## Decisions
- **Decision**: Use a dedicated `AgentConfig` struct for deserialization and configuration.
  - **Rationale**: Keeps `Agent` struct clean and focused on runtime behavior. Matches OpenCode's `Agent.Info`.
- **Decision**: Include `AgentMode` enum (Primary, SubAgent, All) in `AgentConfig`.
  - **Rationale**: Essential for distinguishing agent roles and supporting future sub-agent delegation, ensuring compatibility with OpenCode's architecture.
- **Decision**: Implement `SystemPromptBuilder` as a stateless utility.
  - **Rationale**: Allows easy testing of prompt generation and separation from the `Agent` logic.
- **Decision**: Embed `AgentPermissions` within `AgentConfig`.
  - **Rationale**: Permissions are an inherent part of the agent's definition.

## Risks / Trade-offs
- **Risk**: Complexity in `SystemPromptBuilder` when reading files (e.g., `AGENTS.md`) async.
  - **Mitigation**: Handle file I/O errors gracefully and provide fallbacks.

## Open Questions
- How to handle "Ask" permissions in the current CLI implementation? (Deferred to future UI/CLI interaction updates).
