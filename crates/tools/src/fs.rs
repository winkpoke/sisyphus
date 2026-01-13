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

pub struct ReplaceInFileTool {
    sandbox: Arc<SandboxedPath>,
}

impl ReplaceInFileTool {
    pub fn new(sandbox: Arc<SandboxedPath>) -> Self {
        Self { sandbox }
    }
}

#[async_trait]
impl Tool for ReplaceInFileTool {
    fn name(&self) -> &str {
        "replace_in_file"
    }
    fn description(&self) -> &str {
        "Replace specific text in a file with new text. Finds and replaces occurrences of a string pattern."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to file"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact text to search for and replace. Must match exactly (case-sensitive)"
                },
                "new_string": {
                    "type": "string",
                    "description": "The new text to replace old_string with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "If true, replace all occurrences. If false (default), replace only the first occurrence",
                    "default": false
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid path"))?;
        let old_string = args["old_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid old_string"))?;
        let new_string = args["new_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid new_string"))?;
        let replace_all = args["replace_all"].as_bool().unwrap_or(false);

        if old_string.is_empty() {
            return Err(anyhow::anyhow!("old_string cannot be empty"));
        }

        let path = self.sandbox.join(path_str)?;

        let content = tokio::fs::read_to_string(&path).await?;

        let new_content = if replace_all {
            let count = content.matches(old_string).count();
            if count == 0 {
                return Err(anyhow::anyhow!(
                    "Pattern not found in file: '{}' (0 occurrences)",
                    old_string
                ));
            }
            content.replace(old_string, new_string)
        } else {
            match content.find(old_string) {
                Some(_) => content.replacen(old_string, new_string, 1),
                None => {
                    return Err(anyhow::anyhow!(
                        "Pattern not found in file: '{}'",
                        old_string
                    ));
                }
            }
        };

        tokio::fs::write(&path, new_content).await?;

        Ok("Success".to_string())
    }
}
