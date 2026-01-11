use config::{Config as ConfigLoader, ConfigError, Environment, File, FileFormat};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub llm: LLMConfig,
    pub workspace: WorkspaceConfig,
    pub language: String,
    pub agent: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f64,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub root: String,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        Self::load(None)
    }

    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let mut builder = ConfigLoader::builder()
            .set_default("server.port", 3000)?
            .set_default("server.host", "127.0.0.1")?
            .set_default("llm.provider", "openai")?
            .set_default("llm.model", "gpt-3.5-turbo")?
            .set_default("llm.temperature", 0.2)?
            .set_default("workspace.root", "./workspace")?
            .set_default("language", "en")?;

        // Manual variable substitution
        // If a path is provided, use it. Otherwise check sisyphus.toml
        let target_path = if let Some(p) = config_path {
            if p.exists() {
                Some(p.to_path_buf())
            } else {
                // If the user specified a config file and it doesn't exist, we should probably fail.
                // But ConfigError is from the config crate.
                // For now, let's try to load it and let it fail or handle the error.
                // However, the original logic had manual regex replacement.
                Some(p.to_path_buf())
            }
        } else if Path::new("sisyphus.toml").exists() {
            Some(Path::new("sisyphus.toml").to_path_buf())
        } else {
            None
        };

        if let Some(path) = target_path {
            let content =
                std::fs::read_to_string(&path).map_err(|e| ConfigError::Foreign(Box::new(e)))?;

            let re = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").unwrap();
            let processed_content = re.replace_all(&content, |caps: &regex::Captures| {
                let var_name = &caps[1];
                std::env::var(var_name).unwrap_or_else(|_| {
                    panic!(
                        "Environment variable '{}' not found but required in config file '{}'",
                        var_name,
                        path.display()
                    )
                })
            });

            builder = builder.add_source(File::from_str(&processed_content, FileFormat::Toml));
        } else if config_path.is_none() {
            // Fallback to default search if no explicit path and no sisyphus.toml in cwd
            builder = builder.add_source(File::with_name("sisyphus").required(false));
        }

        let s = builder
            .add_source(Environment::with_prefix("SISYPHUS").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
