use anyhow::{anyhow, Result};
use chrono::Local;
use minijinja::{context, Environment};
use std::env;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

pub fn get_default_system_prompt_template() -> &'static str {
    include_str!("agent/templates/system_prompt.jinja")
}

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub args: String,
    pub argv: Vec<String>,
    pub command: String,
    pub cwd: String,
    pub workspace_root: String,
}

#[derive(Debug, Clone)]
pub struct SystemPromptContext {
    pub instructions: String,
    pub custom_rules: Option<String>,
    pub os: String,
    pub cwd: String,
    pub date: String,
    pub workspace_root: String,
}

impl SystemPromptContext {
    pub fn new(
        instructions: String,
        custom_rules: Option<String>,
        os: String,
        cwd: String,
        date: String,
        workspace_root: String,
    ) -> Self {
        Self {
            instructions,
            custom_rules,
            os,
            cwd,
            date,
            workspace_root,
        }
    }

    /// Capture environment state and create a system prompt context.
    /// This replaces the previous PromptSnapshot approach.
    pub fn capture(
        workspace_root: Option<&Path>,
        instructions: String,
        custom_rules: Option<String>,
    ) -> Self {
        let os = env::consts::OS.to_string();
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let date = Local::now().format("%Y-%m-%d").to_string();
        let workspace_root_str = workspace_root
            .and_then(|p| p.to_str())
            .unwrap_or(&cwd)
            .to_string();

        Self {
            instructions,
            custom_rules,
            os,
            cwd,
            date,
            workspace_root: workspace_root_str,
        }
    }
}

impl TemplateContext {
    pub fn new(
        args: String,
        argv: Vec<String>,
        command: String,
        cwd: String,
        workspace_root: String,
    ) -> Self {
        Self {
            args,
            argv,
            command,
            cwd,
            workspace_root,
        }
    }
}

#[derive(Clone)]
pub struct TemplateEngine {
    env: Arc<Environment<'static>>,
    max_length: usize,
}

impl fmt::Debug for TemplateEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TemplateEngine")
            .field("max_length", &self.max_length)
            .finish()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self::with_max_length(100_000)
    }

    pub fn with_max_length(max_length: usize) -> Self {
        Self {
            env: Arc::new(Environment::new()),
            max_length,
        }
    }

    pub fn validate(&self, template: &str) -> Result<()> {
        self.env
            .template_from_str(template)
            .map_err(|e| anyhow!("Template validation error: {}", e))?;
        Ok(())
    }

    pub fn render(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let ctx = context! {
            args => &context.args,
            argv => &context.argv,
            command => &context.command,
            cwd => &context.cwd,
            workspace_root => &context.workspace_root,
        };

        let rendered = self
            .env
            .render_str(template, ctx)
            .map_err(|e| anyhow!("Template rendering error: {}", e))?;

        if rendered.len() > self.max_length {
            return Err(anyhow!(
                "Rendered output exceeds maximum length of {} characters",
                self.max_length
            ));
        }

        Ok(rendered)
    }

    pub fn render_system_prompt(
        &self,
        template: &str,
        context: &SystemPromptContext,
    ) -> Result<String> {
        let custom_rules = context
            .custom_rules
            .as_ref()
            .map(|s| Self::escape_xml(s))
            .unwrap_or_default();

        let ctx = context! {
            instructions => Self::escape_xml(&context.instructions),
            custom_rules => custom_rules,
            os => Self::escape_xml(&context.os),
            cwd => Self::escape_xml(&context.cwd),
            date => Self::escape_xml(&context.date),
            workspace_root => Self::escape_xml(&context.workspace_root),
        };

        let rendered = self
            .env
            .render_str(template, ctx)
            .map_err(|e| anyhow!("Template rendering error: {}", e))?;

        Ok(rendered)
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_context_creation() {
        let ctx = TemplateContext::new(
            "foo bar".to_string(),
            vec!["foo".to_string(), "bar".to_string()],
            "/test".to_string(),
            "/workspace".to_string(),
            "/workspace".to_string(),
        );

        assert_eq!(ctx.args, "foo bar");
        assert_eq!(ctx.argv, vec!["foo", "bar"]);
        assert_eq!(ctx.command, "/test");
    }

    #[test]
    fn test_template_validation_valid() {
        let engine = TemplateEngine::new();
        assert!(engine.validate("{{args}}").is_ok());
        assert!(engine.validate("{{args}} {{command}}").is_ok());
        assert!(engine.validate("{% if args %}{{args}}{% endif %}").is_ok());
    }

    #[test]
    fn test_template_validation_invalid() {
        let engine = TemplateEngine::new();
        assert!(engine.validate("{{args").is_err());
        assert!(engine.validate("{% if args %}").is_err());
    }

    #[test]
    fn test_template_rendering_basic() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "test".to_string(),
            vec!["test".to_string()],
            "/cmd".to_string(),
            "/cwd".to_string(),
            "/workspace".to_string(),
        );

        let result = engine.render("{{args}}", &ctx).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_template_rendering_backward_compatible() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "foo bar".to_string(),
            vec!["foo".to_string(), "bar".to_string()],
            "/test".to_string(),
            "/workspace".to_string(),
            "/workspace".to_string(),
        );

        let result = engine.render("{{args}}", &ctx).unwrap();
        assert_eq!(result, "foo bar");
    }

    #[test]
    fn test_template_rendering_conditional() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "test".to_string(),
            vec!["test".to_string()],
            "/cmd".to_string(),
            "/cwd".to_string(),
            "/workspace".to_string(),
        );

        let result = engine
            .render(
                "{% if args %}args: {{args}}{% else %}no args{% endif %}",
                &ctx,
            )
            .unwrap();
        assert_eq!(result, "args: test");
    }

    #[test]
    fn test_template_rendering_loop() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "a b c".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "/cmd".to_string(),
            "/cwd".to_string(),
            "/workspace".to_string(),
        );

        let result = engine
            .render("{% for arg in argv %}{{arg}} {% endfor %}", &ctx)
            .unwrap();
        assert_eq!(result, "a b c ");
    }

    #[test]
    fn test_template_rendering_max_length() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "test".to_string(),
            vec!["test".to_string()],
            "/cmd".to_string(),
            "/cwd".to_string(),
            "/workspace".to_string(),
        );

        let long_template = "{{args}}".repeat(150000);
        let result = engine.render(&long_template, &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum length"));
    }

    #[test]
    fn test_template_undefined_variable() {
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::new(
            "test".to_string(),
            vec!["test".to_string()],
            "/cmd".to_string(),
            "/cwd".to_string(),
            "/workspace".to_string(),
        );

        let result = engine.render("{{undefined_var}}", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
