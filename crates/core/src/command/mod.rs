pub mod builtins;

use anyhow::Result;
use async_trait::async_trait;
use common::bus::EventBus;
use gray_matter::{engine::YAML, Matter, Pod};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandEffect {
    None,
    ClearHistory,
    NewSession,
    Exit,
}

pub struct CommandOutcome {
    pub output: Option<String>,
    pub effect: CommandEffect,
}

pub struct CommandContext {
    pub session_id: String,
    pub event_bus: Arc<EventBus>,
}

pub type CommandArgs = Vec<String>;

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, ctx: &CommandContext, args: CommandArgs) -> Result<CommandOutcome>;
}

pub enum CommandType {
    Builtin(Box<dyn Command>),
    Custom(CommandConfig),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub command_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommandConfig {
    pub description: Option<String>,
    pub template: String,
}

pub struct CommandRegistry {
    commands: HashMap<String, CommandType>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register_builtin(&mut self, command: Box<dyn Command>) {
        self.commands.insert(
            command.name().to_string(),
            CommandType::Builtin(command),
        );
    }

    pub fn register_custom(&mut self, name: &str, config: CommandConfig) {
        self.commands
            .insert(name.to_string(), CommandType::Custom(config));
    }

    pub fn get(&self, name: &str) -> Option<&CommandType> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<CommandInfo> {
        let mut list = self
            .commands
            .iter()
            .map(|(name, cmd)| {
                let (desc, type_) = match cmd {
                    CommandType::Builtin(c) => (c.description().to_string(), "builtin"),
                    CommandType::Custom(config) => {
                        (config.description.clone().unwrap_or_default(), "custom")
                    }
                };
                CommandInfo {
                    name: name.clone(),
                    description: desc,
                    command_type: type_.to_string(),
                }
            })
            .collect::<Vec<_>>();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        if !dir.exists() {
            // It's okay if the directory doesn't exist, just log it or return Ok
            warn!("Command directory not found: {:?}", dir);
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                if name.is_empty() {
                    continue;
                }

                // Validate command name (no whitespace)
                if name.contains(char::is_whitespace) {
                    warn!(
                        "Skipping command file with invalid name (contains whitespace): {:?}",
                        path
                    );
                    continue;
                }

                // Load file content
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read command file {:?}: {}", path, e);
                        continue;
                    }
                };

                // Parse frontmatter
                let matter = Matter::<YAML>::new();
                let parsed: gray_matter::ParsedEntity<Pod> = match matter.parse(&content) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to parse markdown {:?}: {}", path, e);
                        continue;
                    }
                };

                if let Some(data) = parsed.data {
                    #[derive(Deserialize)]
                    struct FrontMatter {
                        description: Option<String>,
                    }

                    // gray_matter deserializes Pod to T.
                    let fm: FrontMatter = match data.deserialize() {
                        Ok(fm) => fm,
                        Err(e) => {
                            warn!("Failed to deserialize frontmatter in {:?}: {}", path, e);
                            continue;
                        }
                    };

                    let config = CommandConfig {
                        description: fm.description,
                        template: parsed.content,
                    };

                    let command_name = format!("/{}", name);
                    self.register_custom(&command_name, config);
                    info!("Loaded custom command: {}", command_name);
                } else {
                    warn!("No frontmatter found in {:?}", path);
                }
            }
        }
        Ok(())
    }
}
