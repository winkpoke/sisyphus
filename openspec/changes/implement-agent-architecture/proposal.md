# Change: Implement Agent Architecture

## Why
The current Agent implementation in Sisyphus is minimal and lacks the robust configuration, permission management, and dynamic context handling capabilities present in OpenCode. To fully port OpenCode's core functionality, we need a structured Agent architecture that supports these features.

## What Changes
- Introduce `AgentConfig` and `AgentPermissions` structures to handle agent metadata and security rules.
- Implement `SystemPromptBuilder` to dynamically generate system prompts with environment context and custom rules.
- Refactor the `Agent` struct to separate configuration from runtime state and utilize the new config/prompt systems.
- Create a new `agent-core` capability to track these requirements.

## Impact
- **New Capability**: `agent-core`
- **Affected Code**:
  - `crates/core/src/agent.rs` (Refactor)
  - `crates/core/src/agent/config.rs` (New)
  - `crates/core/src/agent/prompt.rs` (New)
  - `crates/core/src/lib.rs` (Update)
