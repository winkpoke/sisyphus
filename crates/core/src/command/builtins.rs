use super::{Command, CommandArgs, CommandContext, CommandOutcome};
use anyhow::Result;
use async_trait::async_trait;
use common::bus::SystemEvent;
use std::env;
use std::path::Path;

pub struct HelpCommand;

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    fn description(&self) -> &str {
        "Show this help message"
    }
    async fn execute(&self, ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        let mut output = String::new();
        output.push_str("Available commands:\n\n");
        let list = ctx.registry.list();
        // Determine max width for alignment
        let max_len = list.iter().map(|c| c.name.len()).max().unwrap_or(0);

        for cmd in list {
            output.push_str(&format!(
                "  {:<width$} {}\n",
                cmd.name,
                cmd.description,
                width = max_len + 2
            ));
        }
        Ok(CommandOutcome {
            output: Some(output),
        })
    }
}

pub struct CdCommand;

#[async_trait]
impl Command for CdCommand {
    fn name(&self) -> &str {
        "cd"
    }
    fn description(&self) -> &str {
        "Change current working directory"
    }
    async fn execute(&self, ctx: &CommandContext, args: CommandArgs) -> Result<CommandOutcome> {
        let path_str = if args.is_empty() {
            // Default to home directory if available, else root
            dirs::home_dir()
                .map(|p: std::path::PathBuf| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string())
        } else {
            args[0].clone()
        };

        let path = Path::new(&path_str);
        if let Err(e) = env::set_current_dir(path) {
            return Ok(CommandOutcome {
                output: Some(format!("Failed to change directory: {}", e)),
            });
        }

        let new_cwd = env::current_dir()?.to_string_lossy().to_string();

        // Publish directory changed event
        ctx.event_bus.publish(SystemEvent::DirectoryChanged {
            path: new_cwd.clone(),
        });

        Ok(CommandOutcome {
            output: Some(format!("Changed directory to: {}", new_cwd)),
        })
    }
}
