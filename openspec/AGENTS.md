# OpenSpec Instructions

Instructions for AI coding assistants using OpenSpec for spec-driven development.

## TL;DR Quick Checklist

- Search: `openspec spec list --long`, `openspec list`
- Pick unique `change-id`: kebab-case, verb-led (`add-`, `update-`, `remove-`, `refactor-`)
- Scaffold: `proposal.md`, `tasks.md`, `design.md` (only if needed), and delta specs
- Write deltas: `## ADDED|MODIFIED|REMOVED|RENAMED Requirements` with `#### Scenario:` per requirement
- Validate: `openspec validate <id> --strict` and fix issues
- Request approval: Do not start implementation until proposal is approved

## Three-Stage Workflow

### Stage 1: Creating Changes

Create proposal when adding features, breaking changes, architecture changes, performance/security updates.

Skip proposal for: bug fixes (restore intended behavior), typos/format, non-breaking dependency updates, config changes, tests for existing behavior.

**Workflow:**
1. Review `openspec/project.md`, `openspec list`, `openspec list --specs`
2. Choose unique verb-led `change-id` and scaffold under `openspec/changes/<id>/`
3. Draft spec deltas using `## ADDED|MODIFIED|REMOVED Requirements` with scenarios
4. Run `openspec validate <id> --strict` and resolve issues

### Stage 2: Implementing Changes

Track these steps as TODOs:
1. Read `proposal.md`, `design.md` (if exists), `tasks.md`
2. Implement tasks sequentially
3. Update checklist: set every task to `- [x]` after completion
4. **Approval gate**: Do not start implementation until proposal is reviewed and approved

### Stage 3: Archiving Changes

After deployment, create separate PR to:
- Move `changes/[name]/` → `changes/archive/YYYY-MM-DD-[name]/`
- Update `specs/` if capabilities changed
- Use `openspec archive <change-id> --skip-specs --yes` for tooling-only changes
- Run `openspec validate --strict` to confirm the archived change passes checks

## CLI Essentials

```bash
openspec list                  # List active changes
openspec list --specs          # List specifications
openspec show [item]           # Display change or spec
openspec validate [item]       # Validate changes or specs
openspec archive <change-id> --yes   # Archive after deployment
openspec show [change] --json --deltas-only  # Debug delta parsing
```

## Directory Structure

```
openspec/
├── project.md              # Project conventions
├── specs/                  # Current truth - what IS built
│   └── [capability]/       # Single focused capability
│       ├── spec.md         # Requirements and scenarios
│       └── design.md       # Technical patterns (optional)
├── changes/                # Proposals - what SHOULD change
│   ├── [change-name]/
│   │   ├── proposal.md     # Why, what, impact
│   │   ├── tasks.md        # Implementation checklist
│   │   ├── design.md       # Technical decisions (optional)
│   │   └── specs/          # Delta changes
│   │       └── [capability]/
│   │           └── spec.md # ADDED/MODIFIED/REMOVED
│   └── archive/            # Completed changes
```

## Spec File Format

### Critical: Scenario Formatting

**CORRECT** (use #### headers):
```markdown
#### Scenario: User login success
- **WHEN** valid credentials provided
- **THEN** return JWT token
```

**WRONG** (don't use bullets or bold):
```markdown
- **Scenario: User login**  ❌
**Scenario**: User login     ❌
### Scenario: User login      ❌
```

Every requirement MUST have at least one scenario.

### Delta Operations

- `## ADDED Requirements` - New capabilities
- `## MODIFIED Requirements` - Changed behavior (paste full updated requirement)
- `## REMOVED Requirements` - Deprecated features
- `## RENAMED Requirements` - Name changes only

#### When to use ADDED vs MODIFIED
- **ADDED**: New capability that can stand alone. Prefer when change is orthogonal.
- **MODIFIED**: Changes behavior/scope of existing requirement. Always paste the FULL requirement block (header + all scenarios).
- **RENAMED**: Name change only. If behavior also changes, use RENAMED + MODIFIED.

## Proposal Structure

```markdown
# Change: [Brief description]

## Why
[1-2 sentences on problem/opportunity]

## What Changes
- [Bullet list]
- [Mark breaking changes with **BREAKING**]

## Impact
- Affected specs: [list capabilities]
- Affected code: [key files/systems]
```

## Design.md (only if needed)

Create if: cross-cutting change, new architectural pattern, new external dependency, security/performance/migration complexity, or ambiguity needing technical decisions.

**Minimal skeleton:**
```markdown
## Context
[Background, constraints, stakeholders]

## Goals / Non-Goals
- Goals: [...]
- Non-Goals: [...]

## Decisions
- Decision: [What and why]
- Alternatives considered: [Options + rationale]

## Risks / Trade-offs
- [Risk] → Mitigation
```

Remember: Specs are truth. Changes are proposals. Keep them in sync.
