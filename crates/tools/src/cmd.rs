use anyhow::Result;
use async_trait::async_trait;
use common::tool::Tool;
use serde_json::{json, Value};
use tokio::process::Command;

pub struct CommandTool;

#[async_trait]
impl Tool for CommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }
    fn description(&self) -> &str {
        "Execute a shell command"
    }
    fn schema(&self) -> Value {
        json!({
             "type": "object",
             "properties": {
                 "command": { "type": "string" }
             },
             "required": ["command"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let cmd = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing command"))?;

        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .arg("-Command")
                .arg(cmd)
                .output()
                .await?
        } else {
            Command::new("sh").arg("-c").arg(cmd).output().await?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr))
    }
}
