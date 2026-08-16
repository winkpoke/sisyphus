//! Rule-based permission resolution engine.
//!
//! Implements the `refactor-permission-system` OpenSpec change:
//! - Rule strings like `Bash(git:*)`, `Read(./src/**/*.rs)`, `WebFetch`
//! - Resolution order: deny → allow → ask → permission-mode fallback
//! - Legacy `edit`/`bash`/`skill` fields keep working when no rules are
//!   configured and the mode is `Default` (100% backward compatibility)
//! - Deny rules are always enforced, even in `BypassPermissions` mode

use serde_json::Value;

use super::config::{AgentPermissions, PermissionLevel, PermissionMode};

/// Outcome of resolving a tool execution against the permission system.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionDecision {
    /// Execute without prompting.
    Allow,
    /// Refuse execution; `reason` is returned to the model as the tool result.
    Deny {
        reason: String,
        matched_rule: Option<String>,
    },
    /// Pause the turn and emit a `PermissionRequest` event.
    Ask { matched_rule: Option<String> },
}

/// Tool categories used by both the legacy field mapping and mode fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// Shell command execution (`execute_command`).
    Bash,
    /// File-mutating operations (`write_file`, `replace_in_file`, `delete_file`).
    Edit,
    /// File-reading operations with a `path` argument (`read_file`).
    Read,
    /// Everything else (skill tools, `glob`, `grep`, custom tools).
    Other,
}

/// Map a concrete tool name to its permission category.
///
/// These names MUST match the `name()` returned by the tool
/// implementations (see `crates/tools/src`).
pub fn tool_category(tool_name: &str) -> ToolCategory {
    match tool_name {
        "execute_command" => ToolCategory::Bash,
        "write_file" | "replace_in_file" | "delete_file" => ToolCategory::Edit,
        "read_file" => ToolCategory::Read,
        _ => ToolCategory::Other,
    }
}

/// Map Claude Code-style rule tool names to concrete Sisyphus tool names.
/// Concrete names pass through unchanged so both spellings work in rules.
fn canonical_tool_name(name: &str) -> &str {
    match name {
        "Bash" => "execute_command",
        "Read" => "read_file",
        "Write" => "write_file",
        "Edit" => "replace_in_file",
        "Delete" => "delete_file",
        other => other,
    }
}

/// A parsed permission rule: `Tool(pattern)` or bare `Tool`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRule {
    /// Canonical (concrete) tool name.
    pub tool: String,
    /// `Some(pattern)` for `Tool(pattern)` rules, `None` for bare `Tool`
    /// rules (match every operation of the tool).
    pub pattern: Option<String>,
}

/// Parse a rule string into its tool and optional pattern.
///
/// Valid forms: `WebFetch`, `Bash(git:*)`, `Read(./src/**/*.rs)`,
/// `Bash()` (empty pattern = wildcard). Returns `None` for malformed input.
pub fn parse_rule(rule: &str) -> Option<ParsedRule> {
    let rule = rule.trim();
    if let Some(open) = rule.find('(') {
        let close = rule.rfind(')')?;
        if close < open || rule[..open].trim().is_empty() {
            return None;
        }
        Some(ParsedRule {
            tool: canonical_tool_name(rule[..open].trim()).to_string(),
            pattern: Some(rule[open + 1..close].trim().to_string()),
        })
    } else if rule.is_empty() {
        None
    } else {
        Some(ParsedRule {
            tool: canonical_tool_name(rule).to_string(),
            pattern: None,
        })
    }
}

/// Normalize a filesystem path for glob comparison: strip a leading `./`
/// so `Read(./src/**)` matches the path `src/main.rs` and vice versa.
fn normalize_path(p: &str) -> &str {
    p.strip_prefix("./").unwrap_or(p)
}

/// Check whether a single rule matches a tool invocation.
pub fn rule_matches(rule: &str, tool_name: &str, args: &Value) -> bool {
    let Some(parsed) = parse_rule(rule) else {
        return false;
    };
    if parsed.tool != canonical_tool_name(tool_name) {
        return false;
    }

    let Some(pattern) = parsed.pattern else {
        return true; // bare tool name: match every operation
    };
    if pattern.is_empty() || pattern == "*" {
        return true; // wildcard: all operations for this tool
    }

    match tool_category(tool_name) {
        // Bash rules use prefix matching: `Bash(git:*)` matches `git push`.
        // The prefix is word-bounded so `Bash(git:*)` does not match `github ...`.
        ToolCategory::Bash => {
            let Some(command) = args.get("command").and_then(Value::as_str) else {
                return false;
            };
            match pattern.strip_suffix(":*") {
                Some(prefix) => command == prefix || command.starts_with(&format!("{prefix} ")),
                None => command == pattern,
            }
        }
        // File operations use glob patterns against the `path` argument.
        ToolCategory::Edit | ToolCategory::Read => {
            let Some(path) = args.get("path").and_then(Value::as_str) else {
                return false;
            };
            glob::Pattern::new(normalize_path(&pattern))
                .map(|p| p.matches(normalize_path(path)))
                .unwrap_or(false)
        }
        // Non-file tools use exact tool-name matching only; the pattern is
        // not applied.
        ToolCategory::Other => true,
    }
}

/// Legacy resolution: exact pre-refactor behavior via `overrides` and the
/// `edit`/`bash`/`skill` category fields.
fn legacy_level(permissions: &AgentPermissions, tool_name: &str) -> PermissionLevel {
    if let Some(level) = permissions.overrides.get(tool_name) {
        return *level;
    }
    match tool_name {
        "execute_command" => permissions.bash,
        "write_file" | "replace_in_file" | "delete_file" => permissions.edit,
        _ => permissions.skill,
    }
}

fn legacy_decision(level: PermissionLevel) -> PermissionDecision {
    match level {
        PermissionLevel::Allow => PermissionDecision::Allow,
        PermissionLevel::Deny => PermissionDecision::Deny {
            reason: "Permission denied: tool execution is set to Deny.".to_string(),
            matched_rule: None,
        },
        PermissionLevel::Ask => PermissionDecision::Ask { matched_rule: None },
    }
}

/// Mode fallback behavior when no rule matched.
fn mode_fallback(mode: PermissionMode, tool_name: &str) -> PermissionDecision {
    match mode {
        // Standard checking: prompt for everything not explicitly allowed.
        PermissionMode::Default => PermissionDecision::Ask { matched_rule: None },
        // Trusted-editing workflow: auto-approve edit-category tools.
        PermissionMode::AcceptEdits => match tool_category(tool_name) {
            ToolCategory::Edit => PermissionDecision::Allow,
            _ => PermissionDecision::Ask { matched_rule: None },
        },
        // Headless/CI: anything not explicitly allowed is denied outright.
        PermissionMode::DontAsk => PermissionDecision::Deny {
            reason: "Permission denied: tool execution requires explicit allow rule.".to_string(),
            matched_rule: None,
        },
        // Bypass is normally handled before fallback; if reached, allow.
        PermissionMode::BypassPermissions => PermissionDecision::Allow,
        // Read-only exploration: no mutations, prompt for the rest.
        PermissionMode::Plan => match tool_category(tool_name) {
            ToolCategory::Edit => PermissionDecision::Deny {
                reason: "Permission denied: Plan mode does not allow write operations.".to_string(),
                matched_rule: None,
            },
            ToolCategory::Bash => PermissionDecision::Deny {
                reason: "Permission denied: Plan mode does not allow command execution."
                    .to_string(),
                matched_rule: None,
            },
            _ => PermissionDecision::Ask { matched_rule: None },
        },
    }
}

/// Whether any rule-based list is configured.
fn has_rules(permissions: &AgentPermissions) -> bool {
    !permissions.allow.is_empty() || !permissions.ask.is_empty() || !permissions.deny.is_empty()
}

/// Whether any legacy field deviates from its default (`Ask`).
fn has_non_default_legacy(permissions: &AgentPermissions) -> bool {
    permissions.edit != PermissionLevel::Ask
        || permissions.bash != PermissionLevel::Ask
        || permissions.skill != PermissionLevel::Ask
        || !permissions.overrides.is_empty()
}

/// Resolve whether a tool invocation is allowed.
///
/// Deterministic order (first match wins):
/// 1. Legacy path when no rules are configured AND mode is `Default`
///    (byte-for-byte pre-refactor behavior, including overrides).
/// 2. Deny rules — always enforced, even under `BypassPermissions`.
/// 3. `BypassPermissions` short-circuit (allow everything not denied).
/// 4. Allow rules.
/// 5. Ask rules.
/// 6. Permission-mode fallback.
pub fn resolve(
    permissions: &AgentPermissions,
    tool_name: &str,
    args: &Value,
) -> PermissionDecision {
    if !has_rules(permissions) && permissions.mode == PermissionMode::Default {
        return legacy_decision(legacy_level(permissions, tool_name));
    }

    // Deny always wins and short-circuits everything else.
    if let Some(rule) = permissions
        .deny
        .iter()
        .find(|r| rule_matches(r, tool_name, args))
    {
        return PermissionDecision::Deny {
            reason: format!("Permission denied: matched deny rule '{rule}'."),
            matched_rule: Some(rule.clone()),
        };
    }

    // Bypass skips allow/ask/mode checks; deny rules above still apply.
    if permissions.mode == PermissionMode::BypassPermissions {
        return PermissionDecision::Allow;
    }

    if let Some(_rule) = permissions
        .allow
        .iter()
        .find(|r| rule_matches(r, tool_name, args))
    {
        return PermissionDecision::Allow;
    }

    if let Some(rule) = permissions
        .ask
        .iter()
        .find(|r| rule_matches(r, tool_name, args))
    {
        return PermissionDecision::Ask {
            matched_rule: Some(rule.clone()),
        };
    }

    mode_fallback(permissions.mode, tool_name)
}

/// Log a deprecation warning when legacy fields are configured alongside
/// rules (the legacy fields are ignored in that case).
pub fn warn_legacy_ignored(permissions: &AgentPermissions) {
    if has_rules(permissions) && has_non_default_legacy(permissions) {
        tracing::warn!(
            "Legacy edit/bash/skill permissions ignored when allow/ask/deny rules are present. \
             See docs/permissions-migration.md for converting them to rules."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perms() -> AgentPermissions {
        AgentPermissions::default()
    }

    fn args_command(cmd: &str) -> Value {
        serde_json::json!({ "command": cmd })
    }

    fn args_path(path: &str) -> Value {
        serde_json::json!({ "path": path })
    }

    // ---- Rule parsing (task 2.1) ----

    #[test]
    fn parse_bare_tool_name() {
        assert_eq!(
            parse_rule("WebFetch"),
            Some(ParsedRule {
                tool: "WebFetch".to_string(),
                pattern: None
            })
        );
    }

    #[test]
    fn parse_tool_with_pattern_is_canonicalized() {
        assert_eq!(
            parse_rule("Bash(git:*)"),
            Some(ParsedRule {
                tool: "execute_command".to_string(),
                pattern: Some("git:*".to_string())
            })
        );
        assert_eq!(
            parse_rule("Read(./src/**/*.rs)"),
            Some(ParsedRule {
                tool: "read_file".to_string(),
                pattern: Some("./src/**/*.rs".to_string())
            })
        );
    }

    #[test]
    fn parse_accepts_both_tool_spelling_conventions() {
        assert_eq!(parse_rule("Write(x)").unwrap().tool, "write_file");
        assert_eq!(parse_rule("write_file(x)").unwrap().tool, "write_file");
        assert_eq!(parse_rule("Edit(x)").unwrap().tool, "replace_in_file");
        assert_eq!(parse_rule("Delete(x)").unwrap().tool, "delete_file");
    }

    #[test]
    fn parse_rejects_malformed_rules() {
        assert_eq!(parse_rule(""), None);
        assert_eq!(parse_rule("(pattern)"), None);
        assert_eq!(parse_rule("Bash(git:*"), None); // unterminated
    }

    // ---- Bash prefix matching (task 2.3) ----

    #[test]
    fn bash_prefix_matching() {
        assert!(rule_matches(
            "Bash(git:*)",
            "execute_command",
            &args_command("git push origin main")
        ));
        assert!(rule_matches(
            "Bash(git:*)",
            "execute_command",
            &args_command("git")
        ));
        assert!(!rule_matches(
            "Bash(git:*)",
            "execute_command",
            &args_command("npm install")
        ));
        // Word boundary: git:* must not match "github ..."
        assert!(!rule_matches(
            "Bash(git:*)",
            "execute_command",
            &args_command("github clone")
        ));
        // Multi-word prefixes
        assert!(rule_matches(
            "Bash(npm install:*)",
            "execute_command",
            &args_command("npm install lodash")
        ));
        assert!(!rule_matches(
            "Bash(npm install:*)",
            "execute_command",
            &args_command("npm run build")
        ));
    }

    #[test]
    fn bash_exact_match_without_wildcard() {
        assert!(rule_matches(
            "Bash(cargo build)",
            "execute_command",
            &args_command("cargo build")
        ));
        assert!(!rule_matches(
            "Bash(cargo build)",
            "execute_command",
            &args_command("cargo build --release")
        ));
    }

    #[test]
    fn bash_wildcard_allows_any_command() {
        assert!(rule_matches(
            "Bash(*)",
            "execute_command",
            &args_command("rm -rf /")
        ));
        assert!(rule_matches(
            "Bash()",
            "execute_command",
            &args_command("echo hi")
        ));
        assert!(rule_matches(
            "execute_command",
            "execute_command",
            &args_command("anything")
        ));
    }

    // ---- File glob matching (task 2.2) ----

    #[test]
    fn file_glob_matching() {
        assert!(rule_matches(
            "Read(./src/**/*.rs)",
            "read_file",
            &args_path("./src/agent/config.rs")
        ));
        assert!(rule_matches(
            "Read(src/**/*.rs)",
            "read_file",
            &args_path("./src/agent/config.rs")
        ));
        assert!(rule_matches(
            "Read(**/*.env)",
            "read_file",
            &args_path(".env")
        ));
        assert!(!rule_matches(
            "Read(./src/**)",
            "read_file",
            &args_path("./docs/README.md")
        ));
        assert!(rule_matches(
            "Read(./.env)",
            "read_file",
            &args_path(".env")
        ));
    }

    #[test]
    fn write_glob_restricted_to_pattern() {
        assert!(rule_matches(
            "Write(./src/**)",
            "write_file",
            &args_path("./src/main.rs")
        ));
        assert!(!rule_matches(
            "Write(./src/**)",
            "write_file",
            &args_path("./README.md")
        ));
    }

    // ---- Non-file tools: exact name match only (spec) ----

    #[test]
    fn non_file_tools_match_by_name_only() {
        assert!(rule_matches(
            "WebFetch",
            "WebFetch",
            &serde_json::json!({ "url": "https://x" })
        ));
        assert!(rule_matches(
            "grep",
            "grep",
            &serde_json::json!({ "pattern": "x" })
        ));
        assert!(!rule_matches("WebFetch", "grep", &serde_json::json!({})));
    }

    #[test]
    fn rules_do_not_leak_across_tools() {
        assert!(!rule_matches(
            "Bash(git:*)",
            "write_file",
            &args_command("git push")
        ));
        assert!(!rule_matches(
            "Read(**)",
            "execute_command",
            &args_command("git push")
        ));
    }

    // ---- Resolution engine (tasks 3.1, 3.2) ----

    #[test]
    fn deny_takes_precedence_over_allow() {
        let mut p = perms();
        p.deny = vec!["Bash(rm -rf:*)".to_string()];
        p.allow = vec!["Bash(rm:*)".to_string()];
        assert!(matches!(
            resolve(&p, "execute_command", &args_command("rm -rf /tmp")),
            PermissionDecision::Deny { .. }
        ));
    }

    #[test]
    fn deny_wins_even_in_bypass_mode() {
        let mut p = perms();
        p.mode = PermissionMode::BypassPermissions;
        p.deny = vec!["Read(./.env)".to_string()];
        p.allow = vec!["Read(./.env)".to_string()];
        assert!(matches!(
            resolve(&p, "read_file", &args_path(".env")),
            PermissionDecision::Deny { .. }
        ));
    }

    #[test]
    fn bypass_mode_allows_unddenied_operations() {
        let mut p = perms();
        p.mode = PermissionMode::BypassPermissions;
        p.deny = vec!["Read(./.env)".to_string()];
        assert_eq!(
            resolve(&p, "write_file", &args_path("src/main.rs")),
            PermissionDecision::Allow
        );
        assert_eq!(
            resolve(&p, "execute_command", &args_command("rm -rf /")),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn allow_rule_skips_prompting() {
        let mut p = perms();
        p.mode = PermissionMode::Default;
        p.allow = vec!["Write(./src/**)".to_string()];
        assert_eq!(
            resolve(&p, "write_file", &args_path("./src/main.rs")),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn ask_rule_prompts_even_when_mode_would_allow() {
        let mut p = perms();
        p.mode = PermissionMode::AcceptEdits;
        p.ask = vec!["Bash(npm install:*)".to_string()];
        assert_eq!(
            resolve(&p, "execute_command", &args_command("npm install lodash")),
            PermissionDecision::Ask {
                matched_rule: Some("Bash(npm install:*)".to_string())
            }
        );
    }

    #[test]
    fn no_match_falls_back_to_mode() {
        let mut p = perms();
        p.mode = PermissionMode::AcceptEdits;
        assert_eq!(
            resolve(&p, "write_file", &args_path("anywhere/x")),
            PermissionDecision::Allow
        );
        assert!(matches!(
            resolve(&p, "execute_command", &args_command("ls")),
            PermissionDecision::Ask { matched_rule: None }
        ));
    }

    #[test]
    fn default_mode_prompts_for_everything_without_rules() {
        let p = perms();
        // no rules configured: default mode asks for everything
        assert!(matches!(
            resolve(&p, "read_file", &args_path("x")),
            PermissionDecision::Ask { matched_rule: None }
        ));
    }

    #[test]
    fn accept_edits_auto_approves_edit_tools_only() {
        let mut p = perms();
        p.mode = PermissionMode::AcceptEdits;
        assert_eq!(
            resolve(&p, "write_file", &args_path("x")),
            PermissionDecision::Allow
        );
        assert_eq!(
            resolve(&p, "replace_in_file", &args_path("x")),
            PermissionDecision::Allow
        );
        assert_eq!(
            resolve(&p, "delete_file", &args_path("x")),
            PermissionDecision::Allow
        );
        assert!(matches!(
            resolve(&p, "execute_command", &args_command("ls")),
            PermissionDecision::Ask { .. }
        ));
        assert!(matches!(
            resolve(&p, "read_file", &args_path("x")),
            PermissionDecision::Ask { .. }
        ));
    }

    #[test]
    fn dont_ask_denies_unallowed_with_spec_message() {
        let mut p = perms();
        p.mode = PermissionMode::DontAsk;
        p.allow = vec!["Bash(git:*)".to_string()];
        assert_eq!(
            resolve(&p, "execute_command", &args_command("git diff")),
            PermissionDecision::Allow
        );
        assert_eq!(
            resolve(&p, "execute_command", &args_command("npm install")),
            PermissionDecision::Deny {
                reason: "Permission denied: tool execution requires explicit allow rule."
                    .to_string(),
                matched_rule: None,
            }
        );
    }

    #[test]
    fn plan_mode_denies_writes_and_commands_prompts_reads() {
        let mut p = perms();
        p.mode = PermissionMode::Plan;
        assert_eq!(
            resolve(&p, "write_file", &args_path("x")),
            PermissionDecision::Deny {
                reason: "Permission denied: Plan mode does not allow write operations.".to_string(),
                matched_rule: None,
            }
        );
        assert!(matches!(
            resolve(&p, "read_file", &args_path("x")),
            PermissionDecision::Ask { .. }
        ));
        assert!(matches!(
            resolve(&p, "execute_command", &args_command("ls")),
            PermissionDecision::Deny { .. }
        ));
    }

    // ---- Backward compatibility (task 3.3) ----

    #[test]
    fn legacy_fields_used_when_rules_empty_and_mode_default() {
        let mut p = perms();
        p.edit = PermissionLevel::Allow;
        p.bash = PermissionLevel::Ask;
        p.skill = PermissionLevel::Deny;
        assert_eq!(
            resolve(&p, "write_file", &args_path("x")),
            PermissionDecision::Allow
        );
        assert!(matches!(
            resolve(&p, "execute_command", &args_command("ls")),
            PermissionDecision::Ask { .. }
        ));
        assert!(matches!(
            resolve(&p, "read_file", &args_path("x")),
            PermissionDecision::Deny { .. }
        ));
        // Deny message matches the pre-refactor string exactly.
        assert_eq!(
            resolve(&p, "read_file", &args_path("x")),
            PermissionDecision::Deny {
                reason: "Permission denied: tool execution is set to Deny.".to_string(),
                matched_rule: None,
            }
        );
    }

    #[test]
    fn legacy_overrides_still_work_in_legacy_mode() {
        let mut p = perms();
        p.overrides
            .insert("custom_tool".to_string(), PermissionLevel::Allow);
        assert_eq!(
            resolve(&p, "custom_tool", &serde_json::json!({})),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn rules_present_ignore_legacy_fields() {
        let mut p = perms();
        p.edit = PermissionLevel::Allow;
        p.bash = PermissionLevel::Allow;
        p.allow = vec!["Bash(git:*)".to_string()];
        // No rule matches write_file → Default mode fallback asks,
        // even though legacy edit=Allow would have allowed it.
        assert!(matches!(
            resolve(&p, "write_file", &args_path("x")),
            PermissionDecision::Ask { .. }
        ));
        assert_eq!(
            resolve(&p, "execute_command", &args_command("git status")),
            PermissionDecision::Allow
        );
        assert!(has_non_default_legacy(&p));
    }

    #[test]
    fn non_default_mode_applies_even_with_empty_rules() {
        // Plan mode must take effect when rules are empty but mode != Default
        // (e.g. the built-in plan agent).
        let mut p = perms();
        p.mode = PermissionMode::Plan;
        assert!(matches!(
            resolve(&p, "write_file", &args_path("x")),
            PermissionDecision::Deny { .. }
        ));
    }

    #[test]
    fn deprecation_warning_fires_only_when_needed() {
        let mut p = perms();
        p.allow = vec!["Bash(git:*)".to_string()];
        assert!(
            !has_non_default_legacy(&p),
            "all-default legacy fields: no warning"
        );

        p.bash = PermissionLevel::Allow;
        assert!(
            has_non_default_legacy(&p),
            "non-default legacy + rules: warn"
        );

        let mut legacy_only = perms();
        legacy_only.bash = PermissionLevel::Allow;
        // Legacy-only configs (no rules) never warn, even with non-default
        // legacy fields: warn_legacy_ignored requires rules to be present.
        assert!(has_non_default_legacy(&legacy_only));
        assert!(!has_rules(&legacy_only));
    }
}
