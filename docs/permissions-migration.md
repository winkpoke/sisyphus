# Permissions Migration Guide

Sisyphus supports two permission configuration systems:

1. **Legacy** — the original `edit` / `bash` / `skill` three-level
   categorization
2. **Rule-based** — `allow` / `ask` / `deny` lists with pattern matching
   plus a permission `mode`

This guide explains how both work, when each applies, and how to migrate.

## When does each system apply?

The resolution order is deterministic:

- If **no rules** are configured (`allow`, `ask`, `deny` all empty)
  **and** `mode` is `Default` → the **legacy** system is used. This is
  byte-for-byte the pre-refactor behavior, including per-tool
  `overrides`.
- Otherwise the **rule-based** system is used (see below). Legacy fields
  are **ignored** in this case, and a deprecation warning is logged if
  any legacy field is non-default.

## Rule-based system

### Permission modes

The `mode` sets the default behavior when no rule matches a tool
invocation:

| Mode                | Behavior (unmatched tools) |
| ------------------- | -------------------------- |
| `Default`           | Ask (prompt for approval)  |
| `AcceptEdits`       | Allow edits; ask otherwise |
| `DontAsk`           | Deny unless allowed        |
| `BypassPermissions` | Allow all but deny rules   |
| `Plan`              | Deny writes; ask for reads |

`AcceptEdits` covers `write_file`, `replace_in_file`, and
`delete_file`. `BypassPermissions` still enforces deny rules, which are
always checked first.

### Resolution order

First match wins:

1. **Deny rules** — always checked first, even under
   `BypassPermissions`
2. **`BypassPermissions`** short-circuit (allow if reached)
3. **Allow rules**
4. **Ask rules**
5. **Mode fallback** (table above)

### Rule syntax

```text
ToolName                  # every operation of the tool
ToolName(pattern)         # pattern-scoped rule
ToolName(*) or ToolName() # wildcard: every operation of the tool
```

Tool names accept both Claude Code-style spellings and Sisyphus tool
names:

| Rule spelling                        | Matches tool      |
| ------------------------------------ | ----------------- |
| `Bash(...)` / `execute_command(...)` | `execute_command` |
| `Read(...)` / `read_file(...)`       | `read_file`       |
| `Write(...)` / `write_file(...)`     | `write_file`      |
| `Edit(...)` / `replace_in_file(...)` | `replace_in_file` |
| `Delete(...)` / `delete_file(...)`   | `delete_file`     |
| anything else                        | exact name match  |

The "anything else" row matches by exact tool name (e.g. `glob`,
`grep`, `WebFetch`).

### Pattern semantics per tool category

- **Bash commands — prefix matching:** `Bash(git:*)` matches `git` and
  any command starting with `git` (word-bounded, so it does *not*
  match `github ...`). Without the `:*` suffix the rule is an exact
  full-command match: `Bash(cargo build)`.
- **File operations — glob patterns:** `Read(./src/**/*.rs)` matches
  any `.rs` file under `src/`. A leading `./` is optional on both
  sides.
- **Other tools — exact name match:** the pattern is ignored;
  `WebFetch` matches every `WebFetch` call.

## Migration examples

### Allow all edits, ask for everything else (old way)

Legacy:

```toml
[agent.permissions]
edit = "Allow"
bash = "Ask"
skill = "Ask"
```

Rule-based:

```toml
[agent.permissions]
mode = "AcceptEdits"
```

### Allow git commands but deny destructive ones

Legacy (impossible — `bash` is a single coarse level):

```toml
[agent.permissions]
bash = "Ask"
```

Rule-based:

```toml
[agent.permissions]
mode = "DontAsk"
allow = [
  "Bash(git:*)",
  "Bash(cargo:*)",
  "Read",
  "glob",
  "grep",
]
deny = [
  "Bash(rm -rf:*)",
  "Read(./.env)",
  "Read(./secrets/**)",
]
```

### Read-only exploration agent

Legacy approximation (deny whole categories):

```toml
[agent.permissions]
edit = "Deny"
bash = "Deny"
skill = "Allow"
```

Rule-based (read-only mode with explicit read allows):

```toml
[agent.permissions]
mode = "Plan"
allow = ["read_file", "glob", "grep"]
```

### Per-tool overrides (old way) to exact-name allow rules

Legacy:

```toml
[agent.permissions.overrides]
run_tests = "Allow"
```

Rule-based:

```toml
[agent.permissions]
allow = ["run_tests"]
```

## FAQ

**Do deny rules apply in `BypassPermissions` mode?**

Yes. Deny rules are always evaluated first and always win. Bypass only
skips allow/ask checks and the mode fallback. Bypass mode is still
dangerous: anything not explicitly denied executes without prompting.

**What happens to my legacy config if I add a single rule?**

The whole legacy system stops applying for that agent (a warning is
logged if legacy fields are non-default). Convert your legacy levels to
rules at the same time — see the examples above.

**Can I mix both systems?**

They never combine within one agent. Rules empty plus
`mode = "Default"` → legacy; anything else → rules.

**Why does `Plan` mode prompt for reads?**

Plan mode is for untrusted exploration — every unmatched tool prompts.
Add explicit `allow` rules for the read-only tools you trust (as the
built-in plan agent does).
