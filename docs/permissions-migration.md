# Permissions Migration Guide

Sisyphus supports two permission configuration systems:

1. **Legacy** — the original `edit` / `bash` / `skill` three-level categorization
2. **Rule-based** — `allow` / `ask` / `deny` lists with pattern matching plus a permission `mode`

This guide explains how both work, when each applies, and how to migrate.

## When does each system apply?

The resolution order is deterministic:

- If **no rules** are configured (`allow`, `ask`, `deny` all empty) **and** `mode` is `Default`
  → the **legacy** system is used. This is byte-for-byte the pre-refactor behavior,
  including per-tool `overrides`.
- Otherwise the **rule-based** system is used (see below). Legacy fields are
  **ignored** in this case, and a deprecation warning is logged if any legacy
  field is non-default.

## Rule-based system

### Permission modes

The `mode` sets the default behavior when no rule matches a tool invocation:

| Mode | Behavior for unmatched tools |
| ------ | ------------------------------ |
| `Default` | Prompt for approval (Ask) |
| `AcceptEdits` | Auto-approve file edits (`write_file`, `replace_in_file`, `delete_file`); prompt for everything else |
| `DontAsk` | Deny everything not explicitly allowed (for headless/CI use) |
| `BypassPermissions` | Allow everything — **except deny rules, which are always enforced** (dangerous) |
| `Plan` | Deny writes and command execution; prompt for read-only tools (exploration mode) |

### Resolution order

First match wins:

1. **Deny rules** — always checked first, even under `BypassPermissions`
2. **`BypassPermissions`** short-circuit (allow if reached)
3. **Allow rules**
4. **Ask rules**
5. **Mode fallback** (table above)

### Rule syntax

```toml
ToolName                  # every operation of the tool
ToolName(pattern)         # pattern-scoped rule
ToolName(*) or ToolName() # wildcard: every operation of the tool
```toml

Tool names accept both Claude Code-style spellings and Sisyphus tool names:

| Rule spelling | Matches tool |
| --------------- | -------------- |
| `Bash(...)` / `execute_command(...)` | `execute_command` |
| `Read(...)` / `read_file(...)` | `read_file` |
| `Write(...)` / `write_file(...)` | `write_file` |
| `Edit(...)` / `replace_in_file(...)` | `replace_in_file` |
| `Delete(...)` / `delete_file(...)` | `delete_file` |
| anything else | exact tool-name match (e.g. `glob`, `grep`, `WebFetch`) |

### Pattern semantics per tool category

- **Bash commands — prefix matching:** `Bash(git:*)` matches `git` and any command starting with `git` (word-bounded, so it does *not* match `github ...`). Without the `:*` suffix the rule is an exact full-command match: `Bash(cargo build)`.
- **File operations — glob patterns:** `Read(./src/**/*.rs)` matches any `.rs` file under `src/`. A leading `./` is optional on both sides.
- **Other tools — exact name match:** the pattern is ignored; `WebFetch` matches every `WebFetch` call.

## Migration examples

### Allow all edits, ask for everything else (old way)

```toml
# Legacy
[agent.permissions]
edit = "Allow"
bash = "Ask"
skill = "Ask"
```toml

```toml
# Rule-based
[agent.permissions]
mode = "AcceptEdits"
```toml

### Allow git commands but deny destructive ones

```toml
# Legacy: impossible — bash is a single coarse level
[agent.permissions]
bash = "Ask"

# Rule-based
[agent.permissions]
mode = "DontAsk"
allow = ["Bash(git:*)", "Bash(cargo:*)", "Read", "glob", "grep"]
deny = ["Bash(rm -rf:*)", "Read(./.env)", "Read(./secrets/**)"]
```toml

### Read-only exploration agent

```toml
# Legacy approximation: deny whole categories
[agent.permissions]
edit = "Deny"
bash = "Deny"
skill = "Allow"

# Rule-based: read-only mode with explicit read allows
[agent.permissions]
mode = "Plan"
allow = ["read_file", "glob", "grep"]
```toml

### Per-tool overrides (old way) → exact-name allow rules

```toml
# Legacy
[agent.permissions.overrides]
run_tests = "Allow"

# Rule-based
[agent.permissions]
allow = ["run_tests"]
```toml

## FAQ

**Do deny rules apply in `BypassPermissions` mode?**
Yes. Deny rules are always evaluated first and always win. Bypass only skips allow/ask checks and the mode fallback. Bypass mode is still dangerous: anything not explicitly denied executes without prompting.

**What happens to my legacy config if I add a single rule?**
The whole legacy system stops applying for that agent (a warning is logged if legacy fields are non-default). Convert your legacy levels to rules at the same time — see the examples above.

**Can I mix both systems?**
They never combine within one agent. Rules empty + `mode = "Default"` → legacy; anything else → rules.

**Why does `Plan` mode prompt for reads?**
Plan mode is for untrusted exploration — every unmatched tool prompts. Add explicit `allow` rules for the read-only tools you trust (as the built-in plan agent does).
