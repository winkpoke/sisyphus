# Change: Refactor permission system to rule-based with permission modes

## Why
The current permission system uses simple 3-level categorization (edit/bash/skill) which is inflexible and doesn't match Claude Code's capabilities. We need pattern-based rules, permission modes, and fine-grained control to support complex scenarios like subagent isolation, read-only exploration, and trusted environments.

## What Changes
- Add permission modes (Default, AcceptEdits, DontAsk, BypassPermissions, Plan) for different security profiles
- Introduce rule-based permission system with allow/ask/deny lists supporting pattern matching
- Support glob patterns for file operations (Read/Write/Edit)
- Support prefix matching for Bash tool arguments
- Maintain backward compatibility with legacy edit/bash/skill categorization
- Add permission migration path from old to new system

## Impact
- Affected specs: agent-core
- Affected code: crates/core/src/agent/config.rs, crates/core/src/agent.rs
