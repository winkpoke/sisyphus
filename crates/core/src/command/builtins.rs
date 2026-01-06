use super::{Command, CommandArgs, CommandContext, CommandOutcome};
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
