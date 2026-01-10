## Context
Current permission system uses simple categorization (edit/bash/skill) with 3 permission levels (Allow/Deny/Ask). This approach is:
- Too coarse: Cannot distinguish between `git diff` and `rm -rf` under bash
- Inflexible: Cannot allow specific file patterns while denying others
- Non-compliant: Doesn't match Claude Code's rule-based approach
- Limited: No support for permission modes like "read-only plan mode"

Stakeholders:
- Security: Need fine-grained control for production deployments
- Developers: Want easy configuration for common scenarios
- Subagents: Need isolation and restricted capabilities
- Operators: Want to enforce policies without modifying agent code

## Goals / Non-Goals
**Goals:**
- Match Claude Code's permission system capabilities
- Support pattern-based rules for tools and file operations
- Enable permission modes for different operational contexts
- Maintain 100% backward compatibility during migration
- Provide clear migration path from legacy system

**Non-Goals:**
- Implement hooks or dynamic permission evaluation (future scope)
- Add permission templates or inheritance chains (future scope)
- Implement permission UI or interactive editors (future scope)
- Support complex boolean logic in rules (AND/OR operators - future scope)

## Decisions

### Decision 1: Dual-Mode Compatibility
**What:** Support both legacy (edit/bash/skill) and new (allow/ask/deny lists) permission systems simultaneously
**Why:**
- Prevents breaking existing configurations
- Allows gradual migration
- Legacy fields are simple booleans/overrides, new system is rule-based
- Different data structures don't conflict
**Alternatives considered:**
1. Hard break - Reject: Would require all users to update config immediately
2. Migration tool - Reject: Complex to implement correctly, still requires user action
3. Deprecation warnings only - Reject: Doesn't provide new functionality

### Decision 2: Pattern Matching Strategy
**What:** Use glob patterns for files, prefix matching for Bash commands, exact match for other tools
**Why:**
- Glob is industry standard for file patterns (gitignore, .dockerignore)
- Prefix matching is what Claude Code uses for Bash (e.g., "Bash(git:*)" matches all git commands)
- Simple and easy to understand
**Alternatives considered:**
1. Full regex - Reject: Too complex for users, prone to errors
2. No patterns (exact match only) - Reject: Too restrictive
3. Custom DSL - Reject: Over-engineering, not aligned with Claude Code

### Decision 3: Permission Mode Semantics
**What:** Define 5 permission modes with clear, deterministic behavior
**Why:**
- `Default`: Safe baseline with approval prompts
- `AcceptEdits`: Optimized for trusted development workflows
- `DontAsk`: Essential for background/CI/CD tasks
- `BypassPermissions`: Necessary for local development and testing (documented as dangerous)
- `Plan`: Read-only mode for exploration subagents
**Alternatives considered:**
1. Binary (prompt/don't prompt) - Reject: Not flexible enough
2. Configurable modes - Reject: Too complex, unclear semantics
3. Mode inheritance - Reject: Hard to reason about

### Decision 4: Resolution Order
**What:** deny → allow → ask → mode fallback (sequential check, first match wins)
**Why:**
- Security-first: deny rules always take precedence
- Explicit rules override default modes
- Consistent with Claude Code and ACL best practices
- Predictable and deterministic
**Alternatives considered:**
1. Mode first, then rules - Reject: Less secure, rules can't override unsafe modes
2. Score-based system - Reject: Complex, unclear behavior

### Decision 5: Event Bus Enhancement
**What:** Add matched_rule and mode fields to PermissionRequest event
**Why:**
- Clients can show which rule triggered (better UX)
- Debugging: Users understand why permission was requested
- Audit trails: Record decision-making context
**Alternatives considered:**
1. Keep event unchanged - Reject: Poor observability
2. Add separate events - Reject: More complex, unnecessary

## Risks / Trade-offs

**Risk 1: Backward compatibility breaks**
- Impact: Users with existing configs could experience different behavior
- Likelihood: Low - we're maintaining dual-mode support
- Mitigation: Comprehensive test suite, legacy support remains default when rules empty

**Risk 2: Pattern matching complexity**
- Impact: Users may write incorrect patterns
- Likelihood: Medium - glob patterns have edge cases
- Mitigation: Clear documentation, examples in migration guide, error messages for invalid patterns

**Risk 3: Performance**
- Impact: Pattern matching on every tool execution
- Likelihood: Low - pattern matching is fast, tool calls are infrequent
- Mitigation: Pre-compile patterns, use efficient libraries (glob), add benchmarks if needed

**Risk 4: Permission mode confusion**
- Impact: Users choose wrong mode for their use case
- Likelihood: Medium - 5 modes with subtle differences
- Mitigation: Clear documentation, examples for each mode, defaults to safest option (Default)

**Trade-off: Simplicity vs Expressiveness**
- Chose: Expressiveness (rules + modes)
- Sacrificed: Simplicity of 3-field system
- Rationale: Use cases require fine-grained control (e.g., "allow git commands, deny rm")

## Migration Plan

### Phase 1: Dual-Mode Support (Non-breaking)
- Add new data structures alongside legacy fields
- Implement rule-based resolution
- Legacy paths work when rules are empty
- No user action required

### Phase 2: Deprecation Warnings (Non-breaking)
- Emit warnings when legacy fields are used alongside rules
- Log warning: "Legacy edit/bash/skill permissions ignored when rules present"
- Provide migration suggestions in warning messages

### Phase 3: Documentation and Examples (Non-breaking)
- Add migration guide with before/after examples
- Document all 5 permission modes with use cases
- Provide recipe-style examples for common scenarios

### Phase 4: Soft Deprecation (Breaking, planned for future)
- Update defaults to use rules-based system
- Document legacy fields as deprecated in schema
- Continue to support but recommend migration

### Phase 5: Hard Removal (Breaking, future major version)
- Remove edit/bash/skill fields
- Remove backward compatibility layer
- Require migration for all users

## Open Questions
- Should we support wildcard tools in rules (e.g., allow all `Read` operations with "Read")?
  - Decision: Yes, if pattern is empty or "*", allow all operations for that tool
- Should rules support negation (e.g., "Bash(git:*)" but not "Bash(git push:*)")?
  - Decision: No, too complex for v1. Deny rules handle this use case
- Should we support custom tool categories beyond edit/bash/skill?
  - Decision: No, use deny/ask rules for fine-grained control instead
