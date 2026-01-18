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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("test.txt").unwrap();
        tokio::fs::write(&file_path, "Hello, World!").await.unwrap();

        let tool = ReadFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "test.txt"});
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let tool = WriteFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "newfile.txt", "content": "Test content"});
        let result = tool.execute(args).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");

        let file_path = sandbox.join("newfile.txt").unwrap();
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_write_file_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("overwrite.txt").unwrap();
        tokio::fs::write(&file_path, "Original content")
            .await
            .unwrap();

        let tool = WriteFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "overwrite.txt", "content": "New content"});
        let result = tool.execute(args).await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn test_replace_in_file_first_occurrence() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("replace.txt").unwrap();
        tokio::fs::write(&file_path, "foo bar foo bar foo")
            .await
            .unwrap();

        let tool = ReplaceInFileTool::new(Arc::clone(&sandbox));
        let args = json!({
            "path": "replace.txt",
            "old_string": "foo",
            "new_string": "baz",
            "replace_all": false
        });
        let result = tool.execute(args).await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "baz bar foo bar foo");
    }

    #[tokio::test]
    async fn test_replace_in_file_all_occurrences() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("replace.txt").unwrap();
        tokio::fs::write(&file_path, "foo bar foo bar foo")
            .await
            .unwrap();

        let tool = ReplaceInFileTool::new(Arc::clone(&sandbox));
        let args = json!({
            "path": "replace.txt",
            "old_string": "foo",
            "new_string": "baz",
            "replace_all": true
        });
        let result = tool.execute(args).await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "baz bar baz bar baz");
    }

    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let tool = ReadFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "../../../etc/passwd"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_absolute_path_rejection() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let tool = WriteFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "/tmp/test.txt", "content": "content"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_old_string_error() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("test.txt").unwrap();
        tokio::fs::write(&file_path, "some content").await.unwrap();

        let tool = ReplaceInFileTool::new(Arc::clone(&sandbox));
        let args = json!({
            "path": "test.txt",
            "old_string": "",
            "new_string": "new"
        });
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("old_string cannot be empty"));
    }

    #[tokio::test]
    async fn test_pattern_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let file_path = sandbox.join("test.txt").unwrap();
        tokio::fs::write(&file_path, "some content").await.unwrap();

        let tool = ReplaceInFileTool::new(Arc::clone(&sandbox));
        let args = json!({
            "path": "test.txt",
            "old_string": "nonexistent",
            "new_string": "new"
        });
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Pattern not found"));
    }

    #[tokio::test]
    async fn test_missing_path_error() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let tool = ReadFileTool::new(Arc::clone(&sandbox));
        let args = json!({});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing path"));
    }

    #[tokio::test]
    async fn test_missing_content_error() {
        let temp_dir = TempDir::new().unwrap();
        let sandboxed_path = SandboxedPath::new(temp_dir.path().to_path_buf()).unwrap();
        let sandbox = Arc::new(sandboxed_path);

        let tool = WriteFileTool::new(Arc::clone(&sandbox));
        let args = json!({"path": "test.txt"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing content"));
    }
}
