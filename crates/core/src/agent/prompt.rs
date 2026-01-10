use crate::agent::config::AgentConfig;
use crate::template::{get_default_system_prompt_template, SystemPromptContext, TemplateEngine};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SystemPromptBuilder {
    template: String,
    engine: TemplateEngine,
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPromptBuilder {
    pub fn new() -> Self {
        Self {
            template: get_default_system_prompt_template().to_string(),
            engine: TemplateEngine::new(),
        }
    }

    pub fn new_with_template(template: &str) -> Result<Self> {
        let engine = TemplateEngine::new();
        engine.validate(template)?;
        Ok(Self {
            template: template.to_string(),
            engine,
        })
    }

    pub fn from_config(workspace_root: &Path, config: &AgentConfig) -> Result<Self> {
        if let Some(template) = &config.system_prompt_template {
            return Self::new_with_template(template);
        }

        let file_path = workspace_root.join(".sisyphus/templates/system_prompt.jinja");
        if file_path.exists() {
            match std::fs::read_to_string(&file_path) {
                Ok(content) => match Self::new_with_template(&content) {
                    Ok(builder) => {
                        tracing::info!(
                            "Loaded custom system prompt template from {}",
                            file_path.display()
                        );
                        return Ok(builder);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load custom template from {}: {}. Using default template.",
                            file_path.display(),
                            e
                        );
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        "Failed to read template file {}: {}. Using default template.",
                        file_path.display(),
                        e
                    );
                }
            }
        }

        Ok(Self::new())
    }

    pub fn build(&self, config: &AgentConfig, workspace_root: Option<&Path>) -> Result<String> {
        let agents_file = if let Some(root) = workspace_root {
            root.join("AGENTS.md")
        } else {
            std::path::PathBuf::from("AGENTS.md")
        };
        let custom_rules = std::fs::read_to_string(agents_file).ok();

        let context =
            SystemPromptContext::capture(workspace_root, config.instructions.clone(), custom_rules);

        self.engine.render_system_prompt(&self.template, &context)
    }

    pub fn build_from_context(&self, context: &SystemPromptContext) -> Result<String> {
        self.engine.render_system_prompt(&self.template, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_generation() {
        let builder = SystemPromptBuilder::new();
        let config = AgentConfig {
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };

        let prompt = builder.build(&config, None).unwrap();
        assert!(prompt.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_custom_template_override() {
        let custom_template = "<custom>{{instructions}}</custom>";
        let builder = SystemPromptBuilder::new_with_template(custom_template).unwrap();
        let config = AgentConfig {
            instructions: "Custom instructions".to_string(),
            ..Default::default()
        };

        let prompt = builder.build(&config, None).unwrap();
        assert!(prompt.contains("<custom>"));
        assert!(prompt.contains("Custom instructions"));
        assert!(!prompt.contains("<environment>"));
    }

    #[test]
    fn test_template_syntax_error_handling() {
        let invalid_template = "{{instructions";
        let result = SystemPromptBuilder::new_with_template(invalid_template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Template"));
    }

    #[test]
    fn test_xml_style_tags_in_output() {
        let builder = SystemPromptBuilder::new();
        let config = AgentConfig {
            instructions: "You are helpful.".to_string(),
            ..Default::default()
        };

        let prompt = builder.build(&config, None).unwrap();
        assert!(prompt.contains("<instructions>"));
        assert!(prompt.contains("</instructions>"));
        assert!(prompt.contains("<environment>"));
        assert!(prompt.contains("</environment>"));
        assert!(!prompt.contains("<project_rules>"));
    }
}
