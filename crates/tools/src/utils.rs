use anyhow::{anyhow, Result};
use common::path::SandboxedPath;
use std::path::PathBuf;

pub const DENYLIST: &[&str] = &[".git", ".env", ".ssh"];

pub fn validate_path(sandbox: &SandboxedPath, path: &str) -> Result<PathBuf> {
    let full_path = sandbox.join(path)?;
    
    let relative_path = full_path.strip_prefix(sandbox.root())
        .map_err(|_| anyhow!("Path is outside sandbox"))?;
        
    for component in relative_path.components() {
         if let std::path::Component::Normal(name) = component {
             if let Some(name_str) = name.to_str() {
                 if DENYLIST.contains(&name_str) {
                     return Err(anyhow!("Path contains denylisted component: {}", name_str));
                 }
             }
         }
    }

    if full_path.exists() {
         let canonical = full_path.canonicalize()?;
         if !canonical.starts_with(sandbox.root()) {
             return Err(anyhow!("Path resolves outside workspace root"));
         }
    }

    Ok(full_path)
}
