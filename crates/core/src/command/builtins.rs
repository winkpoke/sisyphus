use crate::command::{Command, CommandArgs, CommandContext, CommandEffect, CommandOutcome};
use anyhow::Result;
use async_trait::async_trait;

pub struct HelpCommand;

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "/help"
    }
    fn description(&self) -> &str {
        "Show this help"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("Available commands:\n/help - Show this help\n/exit, /quit - End the session\n/new - Start a new session\n/clear - Clear history".to_string()),
            effect: CommandEffect::None,
        })
    }
}

pub struct ExitCommand;

#[async_trait]
impl Command for ExitCommand {
    fn name(&self) -> &str {
        "/exit"
    }
    fn description(&self) -> &str {
        "End the session"
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
        "/quit"
    }
    fn description(&self) -> &str {
        "End the session"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: None,
            effect: CommandEffect::Exit,
        })
    }
}

pub struct NewSessionCommand;

#[async_trait]
impl Command for NewSessionCommand {
    fn name(&self) -> &str {
        "/new"
    }
    fn description(&self) -> &str {
        "Start a new session"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("New session started.".to_string()),
            effect: CommandEffect::NewSession,
        })
    }
}

pub struct ClearHistoryCommand;

#[async_trait]
impl Command for ClearHistoryCommand {
    fn name(&self) -> &str {
        "/clear"
    }
    fn description(&self) -> &str {
        "Clear history for the current session"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("History cleared.".to_string()),
            effect: CommandEffect::ClearHistory,
        })
    }
}
