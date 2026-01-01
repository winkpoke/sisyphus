use config::{Config as ConfigLoader, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::path::Path;
use regex::Regex;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub llm: LLMConfig,
    pub workspace: WorkspaceConfig,
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
        let mut builder = ConfigLoader::builder()
            .set_default("server.port", 3000)?
            .set_default("server.host", "127.0.0.1")?
            .set_default("llm.provider", "openai")?
            .set_default("llm.model", "gpt-3.5-turbo")?
            .set_default("llm.temperature", 0.7)?
            .set_default("workspace.root", "./workspace")?;

        // Manual variable substitution for sisyphus.toml
        if Path::new("sisyphus.toml").exists() {
            let content = std::fs::read_to_string("sisyphus.toml")
                .map_err(|e| ConfigError::Foreign(Box::new(e)))?;
            
            let re = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").unwrap();
            let processed_content = re.replace_all(&content, |caps: &regex::Captures| {
                let var_name = &caps[1];
                std::env::var(var_name).unwrap_or_else(|_| "".to_string())
            });

            builder = builder.add_source(File::from_str(&processed_content, FileFormat::Toml));
        } else {
             builder = builder.add_source(File::with_name("sisyphus").required(false));
        }

        let s = builder
            .add_source(Environment::with_prefix("SISYPHUS").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
