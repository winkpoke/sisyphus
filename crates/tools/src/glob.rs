use crate::utils::{validate_path, DENYLIST};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::path::SandboxedPath;
use common::tool::Tool;
use globset::{Glob, GlobSetBuilder};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

pub struct GlobTool {
    sandbox: Arc<SandboxedPath>,
}

impl GlobTool {
    pub fn new(sandbox: Arc<SandboxedPath>) -> Self {
        Self { sandbox }
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Discover files using glob patterns"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Glob pattern evaluated against paths relative to path" },
                "path": { "type": "string", "description": "Workspace-relative base directory to search under", "default": "." },
                "include_ignored": { "type": "array", "items": { "type": "string" }, "description": "Glob patterns that selectively re-include otherwise ignored paths", "default": [] },
                "exclude": { "type": "array", "items": { "type": "string" }, "description": "Additional glob patterns to exclude", "default": [] },
                "max_results": { "type": "integer", "description": "Maximum number of file paths returned", "default": 1000 }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let pattern = args["pattern"].as_str().ok_or_else(|| anyhow!("Missing pattern"))?;
        let path_str = args["path"].as_str().unwrap_or(".");
        let include_ignored = args["include_ignored"].as_array().map(|a| {
            a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
        }).unwrap_or_default();
        let exclude = args["exclude"].as_array().map(|a| {
            a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
        }).unwrap_or_default();
        let max_results = args["max_results"].as_u64().unwrap_or(1000) as usize;

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

        // Compile the glob pattern
        let glob = Glob::new(pattern)?.compile_matcher();

        // Compile exclude patterns
        let mut exclude_builder = GlobSetBuilder::new();
        for pat in &exclude {
            exclude_builder.add(Glob::new(pat)?);
        }
        let exclude_set = exclude_builder.build()?;

        let _sandbox = self.sandbox.clone();
        
        // Run blocking IO in spawn_blocking
        let result = tokio::task::spawn_blocking(move || -> Result<(Vec<String>, bool)> {
            let mut builder = WalkBuilder::new(&base_path);
            
            // Configure ignore rules
            builder.git_ignore(true)
                   .require_git(false)
                   .ignore(true)
                   .parents(true)
                   .sort_by_file_path(|a, b| a.cmp(b));

            // Handle include_ignored and exclude using Overrides
            let mut override_builder = OverrideBuilder::new(&base_path);
            
            for pat in include_ignored {
                // To re-include, we use the "!" prefix in overrides
                let p = format!("!{}", pat);
                override_builder.add(&p)?;
            }
            
            // Excludes are handled manually via GlobSet
            
            let overrides = override_builder.build().map_err(|e| anyhow!("Invalid override pattern: {}", e))?;
            builder.overrides(overrides);

            let mut paths = Vec::new();
            let mut truncated = false;
            let mut count = 0;

            for result in builder.build() {
                match result {
                    Ok(entry) => {
                        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                            continue;
                        }

                        let path = entry.path();
                        
                        // Check denylist on path components
                        if let Ok(rel) = path.strip_prefix(&root) {
                            let mut denylisted = false;
                            for comp in rel.components() {
                                if let std::path::Component::Normal(c) = comp {
                                    if DENYLIST.contains(&c.to_str().unwrap_or("")) {
                                        denylisted = true;
                                        break;
                                    }
                                }
                            }
                            if denylisted { continue; }
                        }

                        // Check if path is within sandbox
                        // The base_path check verified the root of the search.
                        
                        // Check pattern
                        if let Ok(rel_path) = path.strip_prefix(&base_path) {
                            if exclude_set.is_match(rel_path) {
                                continue;
                            }

                            if glob.is_match(rel_path) {
                                if count >= max_results {
                                    truncated = true;
                                    break;
                                }
                                
                                if let Ok(workspace_rel) = path.strip_prefix(&root) {
                                    // Normalize separators to /
                                    let normalized = workspace_rel.to_string_lossy().replace('\\', "/");
                                    paths.push(normalized);
                                    count += 1;
                                }
                            }
                        }
                    }
                    Err(_err) => {
                        continue;
                    },
                }
            }
            
            paths.sort(); // Ensure deterministic order
            
            Ok((paths, truncated))
        }).await??;

        Ok(serde_json::to_string(&json!({
            "paths": result.0,
            "truncated": result.1
        }))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_glob_basic() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GlobTool::new(sandbox);

        // Create some files
        File::create(root.join("a.txt"))?;
        std::fs::create_dir(root.join("b"))?;
        File::create(root.join("b/c.rs"))?;

        let args = json!({
            "pattern": "**/*.rs"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        
        let paths = result["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].as_str().unwrap(), "b/c.rs");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_glob_exclude() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GlobTool::new(sandbox);

        File::create(root.join("a.rs"))?;
        File::create(root.join("b.rs"))?;

        let args = json!({
            "pattern": "*.rs",
            "exclude": ["b.rs"]
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        
        let paths = result["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].as_str().unwrap(), "a.rs");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_glob_ignore() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GlobTool::new(sandbox);

        // Create .gitignore
        let mut gitignore = File::create(root.join(".gitignore"))?;
        writeln!(gitignore, "ignored_file.rs")?;
        
        File::create(root.join("ignored_file.rs"))?;
        File::create(root.join("visible.rs"))?;

        let args = json!({
            "pattern": "**/*.rs"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        
        let paths = result["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].as_str().unwrap(), "visible.rs");

        // Now test manual .ignore file which should work
        let mut ignore = File::create(root.join(".ignore"))?;
        writeln!(ignore, "!ignored_file.rs")?;
        
        let args = json!({
            "pattern": "**/*.rs"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        
        let paths = result["paths"].as_array().unwrap();
        // println!("Paths with .ignore: {:?}", paths);
        assert_eq!(paths.len(), 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_glob_denylist() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GlobTool::new(sandbox);

        std::fs::create_dir(root.join(".git"))?;
        File::create(root.join(".git/config"))?;
        File::create(root.join("safe.txt"))?;

        let args = json!({
            "pattern": "**/*"
        });
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        
        let _paths = result["paths"].as_array().unwrap();
        // Should not find .git content even if we asked for **/*
        // .git is usually ignored by default walk, but DENYLIST should catch it even if we include ignored?
        // Let's try to force include .git
        
        let args = json!({
            "pattern": "**/*",
            "include_ignored": [".git/**"]
        });
        // We expect .git files to be NOT returned because of DENYLIST check in loop
        let result = tool.execute(args).await?;
        let result: Value = serde_json::from_str(&result)?;
        let paths = result["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].as_str().unwrap(), "safe.txt");

        // Test explicit path denylist
        let args = json!({
            "pattern": "*",
            "path": ".git"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err()); // Should be blocked by validate_path

        Ok(())
    }

    #[tokio::test]
    async fn test_glob_invalid_override() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        let sandbox = Arc::new(SandboxedPath::new(root)?);
        let tool = GlobTool::new(sandbox);

        let args = json!({
            "pattern": "**/*.rs",
            "include_ignored": ["**/*.rs["]
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid override pattern"));
        
        Ok(())
    }
}
