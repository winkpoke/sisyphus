use crate::utils::{validate_path, DENYLIST};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::path::SandboxedPath;
use common::tool::{ExecutionMode, Tool};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

pub struct GrepTool {
    sandbox: Arc<SandboxedPath>,
}

impl GrepTool {
    pub fn new(sandbox: Arc<SandboxedPath>) -> Self {
        Self { sandbox }
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using a regular expression"
    }

    fn execution_mode(&self) -> ExecutionMode {
        // Read-only content search; safe to run concurrently.
        ExecutionMode::Parallel
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Regular expression to search for" },
                "path": { "type": "string", "description": "Workspace-relative base directory to search under", "default": "." },
                "output_mode": { "type": "string", "enum": ["files_with_matches", "content", "count"], "default": "files_with_matches" },
                "include_ignored": { "type": "array", "items": { "type": "string" }, "description": "Glob patterns that selectively re-include otherwise ignored paths", "default": [] },
                "exclude": { "type": "array", "items": { "type": "string" }, "description": "Additional glob patterns to exclude", "default": [] },
                "max_results": { "type": "integer", "description": "Maximum number of result entries returned", "default": 100 },
                "max_file_bytes": { "type": "integer", "description": "Maximum number of bytes read per file", "default": 1000000 },
                "max_line_length": { "type": "integer", "description": "Maximum line length included in content results", "default": 2000 }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let pattern_str = args["pattern"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing pattern"))?;
        let path_str = args["path"].as_str().unwrap_or(".");
        let output_mode = args["output_mode"].as_str().unwrap_or("files_with_matches");
        let include_ignored = args["include_ignored"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let exclude = args["exclude"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;
        let max_file_bytes = args["max_file_bytes"].as_u64().unwrap_or(1000000);
        let max_line_length = args["max_line_length"].as_u64().unwrap_or(2000) as usize;

        let root = self.sandbox.root().to_path_buf();
        #[cfg(windows)]
        let root = {
            let s = root.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                PathBuf::from(stripped)
            } else {
                root
            }
        };

        let base_path = validate_path(&self.sandbox, path_str)?;
        #[cfg(windows)]
        let base_path = {
            let s = base_path.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                PathBuf::from(stripped)
            } else {
                base_path
            }
        };

        // Compile regex
        let regex = RegexBuilder::new(pattern_str)
            .size_limit(10 * 1024 * 1024) // 10MB limit for compiled regex
            .dfa_size_limit(10 * 1024 * 1024)
            .build()
            .map_err(|e| anyhow!("Invalid regex: {}", e))?;

        let output_mode = output_mode.to_string();
        let output_mode_for_thread = output_mode.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<(Vec<Value>, bool)> {
            let mut builder = WalkBuilder::new(&base_path);
            builder
                .git_ignore(true)
                .require_git(false)
                .ignore(true)
                .parents(true)
                .sort_by_file_path(|a, b| a.cmp(b));

            let mut override_builder = OverrideBuilder::new(&base_path);
            for pat in include_ignored {
                override_builder.add(&format!("!{}", pat))?;
            }
            for pat in exclude {
                override_builder.add(&pat)?;
            }
            let overrides = override_builder
                .build()
                .map_err(|e| anyhow!("Invalid override pattern: {}", e))?;
            builder.overrides(overrides);

            let mut results = Vec::new();
            let mut truncated = false;
            let mut count = 0;

            for result in builder.build() {
                if count >= max_results {
                    truncated = true;
                    break;
                }

                match result {
                    Ok(entry) => {
                        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                            continue;
                        }

                        let path = entry.path();
                        if let Ok(rel) = path.strip_prefix(&root) {
                            let mut denylisted = false;
                            for comp in rel.components() {
                                if matches!(
                                    comp,
                                    std::path::Component::Normal(c)
                                        if DENYLIST.contains(&c.to_str().unwrap_or(""))
                                ) {
                                    denylisted = true;
                                    break;
                                }
                            }
                            if denylisted {
                                continue;
                            }
                        }

                        // Read file with limit
                        let file = match File::open(path) {
                            Ok(f) => f,
                            Err(_) => continue,
                        };

                        let mut buffer = Vec::new();
                        let mut take = file.take(max_file_bytes);
                        if take.read_to_end(&mut buffer).is_err() {
                            continue;
                        }

                        // Check for NUL byte
                        if buffer.contains(&0) {
                            continue;
                        }

                        // Check for valid UTF-8
                        let content = match String::from_utf8(buffer) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let normalized_path = if let Ok(workspace_rel) = path.strip_prefix(&root) {
                            workspace_rel.to_string_lossy().replace('\\', "/")
                        } else {
                            continue;
                        };

                        if output_mode_for_thread == "files_with_matches" {
                            for l in content.lines() {
                                if regex.is_match(l) {
                                    results.push(json!(normalized_path));
                                    count += 1;
                                    break;
                                }
                            }
                        } else if output_mode_for_thread == "count" {
                            let mut file_count = 0;
                            for l in content.lines() {
                                file_count += regex.find_iter(l).count();
                            }
                            if file_count > 0 {
                                results.push(json!({
                                    "path": normalized_path,
                                    "count": file_count
                                }));
                                count += 1;
                            }
                        } else if output_mode_for_thread == "content" {
                            let mut line_num = 0;
                            for l in content.lines() {
                                line_num += 1;
                                for mat in regex.find_iter(l) {
                                    if count >= max_results {
                                        truncated = true;
                                        break;
                                    }

                                    let byte_start = mat.start();
                                    let column = l[..byte_start].chars().count() + 1;

                                    if column > max_line_length {
                                        continue;
                                    }

                                    let text: String = l.chars().take(max_line_length).collect();

                                    results.push(json!({
                                        "path": normalized_path,
                                        "line": line_num,
                                        "column": column,
                                        "text": text
                                    }));
                                    count += 1;
                                }
                                if truncated {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
                if truncated {
                    break;
                }
            }

            Ok((results, truncated))
        })
        .await??;

        Ok(serde_json::to_string(&json!({
            "output_mode": output_mode,
            "results": result.0,
            "truncated": result.1
        }))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_grep_basic() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("a.txt"))?;
        writeln!(f, "hello world")?;
        writeln!(f, "hello universe")?;

        let args = json!({
            "pattern": "hello"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;

        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_str().unwrap(), "a.txt");

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_content() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("a.txt"))?;
        writeln!(f, "hello world")?;
        writeln!(f, "goodbye world")?;
        writeln!(f, "hello universe")?;

        let args = json!({
            "pattern": "hello",
            "output_mode": "content"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;

        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        let r1 = &results[0];
        assert_eq!(r1["path"], "a.txt");
        assert_eq!(r1["line"], 1);
        assert_eq!(r1["text"], "hello world");

        let r2 = &results[1];
        assert_eq!(r2["path"], "a.txt");
        assert_eq!(r2["line"], 3);
        assert_eq!(r2["text"], "hello universe");

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_count() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("a.txt"))?;
        writeln!(f, "foo")?;
        writeln!(f, "foo foo")?;

        let args = json!({
            "pattern": "foo",
            "output_mode": "count"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;

        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["path"], "a.txt");
        assert_eq!(results[0]["count"], 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_max_results() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("a.txt"))?;
        writeln!(f, "match")?;
        writeln!(f, "match")?;
        writeln!(f, "match")?;

        let args = json!({
            "pattern": "match",
            "output_mode": "content",
            "max_results": 2
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;

        assert_eq!(result["truncated"], true);
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_binary_file_skip() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("binary.bin"))?;
        f.write_all(&[0x68, 0x65, 0x6c, 0x6c, 0x00, 0x6f])?; // hell\0o

        let args = json!({
            "pattern": "hell"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_invalid_utf8_skip() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("invalid_utf8.txt"))?;
        f.write_all(&[0x80, 0x81])?; // Invalid start bytes

        let args = json!({
            "pattern": "."
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_utf8_columns() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("utf8.txt"))?;
        // "🦀 hello" -> crab is 4 bytes, 1 char
        writeln!(f, "🦀 hello")?;

        let args = json!({
            "pattern": "hello",
            "output_mode": "content"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let results = result["results"].as_array().unwrap();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r["column"], 3); // 1 (crab) + 1 (space) + 1 = 3.

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_content_truncation() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let mut f = File::create(root.join("long.txt"))?;
        let long_line = "a".repeat(100) + "match";
        writeln!(f, "{}", long_line)?;

        // Case 1: Match within limit
        let args = json!({
            "pattern": "match",
            "output_mode": "content",
            "max_line_length": 200
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r["text"], long_line);

        // Case 2: Match outside limit
        let args = json!({
            "pattern": "match",
            "output_mode": "content",
            "max_line_length": 50
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GrepTool::new(sandbox);

        let args = json!({
            "pattern": "("
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex"));

        Ok(())
    }
}
