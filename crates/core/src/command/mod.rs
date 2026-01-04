pub mod builtins;
pub mod loader;
pub mod parser;

use anyhow::Result;
use async_trait::async_trait;
use common::bus::EventBus;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CommandEffect {
    None,
    ClearHistory,
    NewSession,
    Exit,
    ToggleDebug,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandOutcome {
    pub output: Option<String>,
    pub effect: CommandEffect,
}

pub struct CommandContext<'a> {
    pub session_id: String,
    pub event_bus: Arc<EventBus>,
    pub registry: &'a CommandRegistry,
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
        self.commands
            .insert(command.name().to_string(), CommandType::Builtin(command));
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
}
