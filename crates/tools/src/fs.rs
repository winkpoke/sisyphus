use anyhow::Result;
use async_trait::async_trait;
use common::path::SandboxedPath;
use common::tool::Tool;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct ReadFileTool {
    sandbox: Arc<SandboxedPath>,
}

impl ReadFileTool {
    pub fn new(sandbox: Arc<SandboxedPath>) -> Self {
        Self { sandbox }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read a file from the workspace"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path to file" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let path = self.sandbox.join(path_str)?;
        let content = tokio::fs::read_to_string(path).await?;
        Ok(content)
    }
}

pub struct WriteFileTool {
    sandbox: Arc<SandboxedPath>,
}

impl WriteFileTool {
    pub fn new(sandbox: Arc<SandboxedPath>) -> Self {
        Self { sandbox }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Write content to a file in the workspace"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path to file" },
                "content": { "type": "string", "description": "Content to write" }
            },
            "required": ["path", "content"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let path = self.sandbox.join(path_str)?;

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(path, content).await?;
        Ok("Success".to_string())
    }
}
