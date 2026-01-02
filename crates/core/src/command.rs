use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use serde::Deserialize;
use gray_matter::{Matter, engine::YAML, Pod};
use tracing::{info, warn};
use crate::session::Session;

pub struct AgentContext<'a> {
    pub session: &'a mut Session,
}

pub enum CommandType {
    Builtin {
        handler: Box<dyn Fn(&mut AgentContext, &[String]) -> Result<String> + Send + Sync>,
        description: String,
    },
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

    pub fn register_builtin<F>(&mut self, name: &str, description: &str, f: F)
    where
        F: Fn(&mut AgentContext, &[String]) -> Result<String> + Send + Sync + 'static,
    {
        self.commands.insert(name.to_string(), CommandType::Builtin {
            handler: Box::new(f),
            description: description.to_string(),
        });
    }

    pub fn register_custom(&mut self, name: &str, config: CommandConfig) {
        self.commands.insert(name.to_string(), CommandType::Custom(config));
    }
    
    pub fn get(&self, name: &str) -> Option<&CommandType> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<CommandInfo> {
        let mut list = self.commands.iter().map(|(name, cmd)| {
            let (desc, type_) = match cmd {
                CommandType::Builtin { description, .. } => (description.clone(), "builtin"),
                CommandType::Custom(config) => (config.description.clone().unwrap_or_default(), "custom"),
            };
            CommandInfo {
                name: name.clone(),
                description: desc,
                command_type: type_.to_string(),
            }
        }).collect::<Vec<_>>();
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
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                if name.is_empty() { continue; }

                // Validate command name (no whitespace)
                if name.contains(char::is_whitespace) {
                    warn!("Skipping command file with invalid name (contains whitespace): {:?}", path);
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
