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

#[cfg(test)]
mod tests {
    use super::CommandTool;
    use common::tool::Tool;
    use serde_json::json;

    #[tokio::test]
    async fn test_execute_command_success() {
        let tool = CommandTool;

        let args = json!({ "command": "echo hello" });
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_command_missing_command() {
        let tool = CommandTool;

        let args = json!({});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing command"));
    }

    #[tokio::test]
    async fn test_execute_command_stdout_capture() {
        let tool = CommandTool;

        let args = json!({ "command": "echo stdout test" });
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Stdout:"));
        assert!(output.contains("stdout test"));
    }

    #[tokio::test]
    async fn test_execute_command_stderr_capture() {
        let tool = CommandTool;

        let args = json!({ "command": "sh -c 'echo error >&2'" });
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Stderr:"));
    }

    #[tokio::test]
    async fn test_tool_name() {
        let tool = CommandTool;

        assert_eq!(tool.name(), "execute_command");
    }

    #[tokio::test]
    async fn test_tool_description() {
        let tool = CommandTool;

        assert_eq!(tool.description(), "Execute a shell command");
    }

    #[tokio::test]
    async fn test_tool_schema() {
        let tool = CommandTool;

        let schema = tool.schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["command"].is_object());
        assert_eq!(
            schema["required"]
                .as_array()
                .unwrap()
                .get(0)
                .map(|v| v.as_str()),
            Some(Some("command"))
        );
    }

    #[tokio::test]
    async fn test_complex_command() {
        let tool = CommandTool;

        let args = json!({ "command": "ls -la | head -5" });
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Stdout:"));
    }

    #[tokio::test]
    async fn test_command_with_quotes() {
        let tool = CommandTool;

        let args = json!({ "command": "echo 'quoted string'" });
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("quoted string"));
    }
}
