use super::{Command, CommandArgs, CommandContext, CommandEffect, CommandOutcome};
use anyhow::Result;
use async_trait::async_trait;

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
            output.push_str(&format!("  {:<width$} {}\n", cmd.name, cmd.description, width = max_len + 2));
        }
        Ok(CommandOutcome {
            output: Some(output),
            effect: CommandEffect::None,
        })
    }
}

pub struct ExitCommand;

#[async_trait]
impl Command for ExitCommand {
    fn name(&self) -> &str {
        "exit"
    }
    fn description(&self) -> &str {
        "Exit the application"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: None,
            effect: CommandEffect::Exit,
        })
    }
}

pub struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
    }
    fn description(&self) -> &str {
        "Exit the application"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: None,
            effect: CommandEffect::Exit,
        })
    }
}

pub struct ClearHistoryCommand;

#[async_trait]
impl Command for ClearHistoryCommand {
    fn name(&self) -> &str {
        "clear"
    }
    fn description(&self) -> &str {
        "Clear the chat history"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("History cleared".to_string()),
            effect: CommandEffect::ClearHistory,
        })
    }
}

pub struct NewSessionCommand;

#[async_trait]
impl Command for NewSessionCommand {
    fn name(&self) -> &str {
        "new"
    }
    fn description(&self) -> &str {
        "Start a new session"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("Started new session".to_string()),
            effect: CommandEffect::NewSession,
        })
    }
}

pub struct DebugCommand;

#[async_trait]
impl Command for DebugCommand {
    fn name(&self) -> &str {
        "debug"
    }
    fn description(&self) -> &str {
        "Toggle debug mode"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: None,
            effect: CommandEffect::ToggleDebug,
        })
    }
}
