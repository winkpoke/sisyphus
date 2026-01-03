use crate::agent::config::AgentConfig;
use chrono::Local;
use std::env;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct PromptSnapshot {
    pub os: String,
    pub cwd: String,
    pub date: String,
    pub custom_rules: Option<String>,
}

pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub fn new() -> Self {
        Self
    }

    pub async fn snapshot(workspace_root: Option<&Path>) -> PromptSnapshot {
        let os = env::consts::OS.to_string();
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let date = Local::now().format("%Y-%m-%d").to_string();

        // Try to read AGENTS.md from workspace root or current directory
        let agents_file = if let Some(root) = workspace_root {
            root.join("AGENTS.md")
        } else {
            Path::new("AGENTS.md").to_path_buf()
        };

        let custom_rules = fs::read_to_string(agents_file).await.ok();

        PromptSnapshot {
            os,
            cwd,
            date,
            custom_rules,
        }
    }

    pub fn build(config: &AgentConfig, snapshot: &PromptSnapshot) -> String {
        let env_info = format!(
            "Environment:\n- OS: {}\n- CWD: {}\n- Date: {}",
            snapshot.os, snapshot.cwd, snapshot.date
        );

        let mut parts = vec![config.instructions.clone(), env_info];

        if let Some(rules) = &snapshot.custom_rules {
            parts.push(rules.clone());
        }

        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_generation() {
        let config = AgentConfig {
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };

        let snapshot = PromptSnapshot {
            os: "linux".to_string(),
            cwd: "/tmp".to_string(),
            date: "2023-01-01".to_string(),
            custom_rules: Some("Rule 1".to_string()),
        };

        let prompt = SystemPromptBuilder::build(&config, &snapshot);

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("Environment:"));
        assert!(prompt.contains("OS: linux"));
        assert!(prompt.contains("CWD: /tmp"));
        assert!(prompt.contains("Date: 2023-01-01"));
        assert!(prompt.contains("Rule 1"));
    }
}
