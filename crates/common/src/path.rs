use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SandboxedPath {
    root: PathBuf,
}

impl SandboxedPath {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        // We assume the root exists or we can create it.
        // For safety, let's canonicalize it to resolve symlinks and relative paths.
        let root = root.as_ref();
        if !root.exists() {
            std::fs::create_dir_all(root)?;
        }
        let root = root.canonicalize()?;
        Ok(Self { root })
    }

    /// Safely joins a relative path to the root.
    /// Rejects absolute paths or paths containing `..`.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();

        if path.is_absolute() {
            return Err(anyhow!("Path must be relative, got: {:?}", path));
        }

        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(anyhow!("Traversal (..) not allowed in path: {:?}", path))
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return Err(anyhow!(
                        "Absolute components not allowed in path: {:?}",
                        path
                    ))
                }
                _ => {}
            }
        }

        Ok(self.root.join(path))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
