use crate::agent::config::AgentConfig;
use std::env;
use chrono::Local;

pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn environment() -> String {
        let os = env::consts::OS;
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let date = Local::now().format("%Y-%m-%d").to_string();

        format!(
            "Environment:\n- OS: {}\n- CWD: {}\n- Date: {}",
            os, cwd, date
        )
    }

    pub fn custom_rules() -> String {
        // Placeholder for reading custom rules from a file (e.g. AGENTS.md)
        String::new()
    }

    pub fn build(config: &AgentConfig) -> String {
        let env_info = Self::environment();
        let rules = Self::custom_rules();
        
        let mut parts = vec![
            config.instructions.clone(),
            env_info,
        ];

        if !rules.is_empty() {
            parts.push(rules);
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

        let prompt = SystemPromptBuilder::build(&config);

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("Environment:"));
        assert!(prompt.contains("OS:"));
        assert!(prompt.contains("CWD:"));
        assert!(prompt.contains("Date:"));
    }
}
