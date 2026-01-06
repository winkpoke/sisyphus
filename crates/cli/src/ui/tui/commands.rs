use anyhow::Result;
use async_trait::async_trait;
use sisyphus_core::command::{Command, CommandArgs, CommandContext, CommandOutcome};

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
            output: Some("Exiting...".to_string()),
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
            output: Some("Exiting...".to_string()),
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
        "Clear session history"
    }
    async fn execute(&self, _ctx: &CommandContext, _args: CommandArgs) -> Result<CommandOutcome> {
        Ok(CommandOutcome {
            output: Some("Clearing history...".to_string()),
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
            output: Some("Starting new session...".to_string()),
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
            output: Some("Toggling debug mode...".to_string()),
        })
    }
}
