use super::CommandConfig;
use crate::template::TemplateEngine;
use anyhow::Result;
use gray_matter::{engine::YAML, Matter, Pod};
use serde::Deserialize;
use std::path::Path;
use tracing::warn;

pub struct CommandLoader;

impl CommandLoader {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<(String, CommandConfig)>> {
        let dir = dir.as_ref();
        let mut commands = Vec::new();

        if !dir.exists() {
            warn!("Command directory not found: {:?}", dir);
            return Ok(commands);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                if name.is_empty() {
                    continue;
                }

                // Validate command name (no whitespace)
                if name.contains(char::is_whitespace) {
                    warn!(
                        "Skipping command file with invalid name (contains whitespace): {:?}",
                        path
                    );
                    continue;
                }

                let raw_content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read command file {:?}: {}", path, e);
                        continue;
                    }
                };

                let matter = Matter::<YAML>::new();
                let parsed: gray_matter::ParsedEntity<Pod> = match matter.parse(&raw_content) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to parse markdown {:?}: {}", path, e);
                        continue;
                    }
                };

                let engine = TemplateEngine::new();
                if let Err(e) = engine.validate(&parsed.content) {
                    warn!(
                        "Skipping command file with invalid template body {:?}: {}",
                        path, e
                    );
                    continue;
                }

                if let Some(data) = parsed.data {
                    #[derive(Deserialize)]
                    struct FrontMatter {
                        description: Option<String>,
                    }

                    // gray_matter deserializes Pod to T.
                    let fm: FrontMatter = match data.deserialize() {
                        Ok(fm) => fm,
                        Err(e) => {
                            warn!("Failed to deserialize frontmatter in {:?}: {}", path, e);
                            continue;
                        }
                    };

                    let config = CommandConfig {
                        description: fm.description,
                        template: parsed.content,
                    };

                    let command_name = format!("/{}", name);
                    commands.push((command_name, config));
                } else {
                    warn!("No frontmatter found in {:?}", path);
                }
            }
        }
        Ok(commands)
    }
}
