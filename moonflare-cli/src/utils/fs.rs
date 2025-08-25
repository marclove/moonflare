use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn create_directory_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn is_moonflare_workspace() -> bool {
    Path::new(".moon/workspace.yml").exists() || 
    Path::new("package.json").exists()
}

pub fn get_project_directory(project_type: &str) -> &'static str {
    match project_type {
        "astro" => "sites",
        "react" => "apps", 
        "durable-object" | "worker" => "workers",
        "crate" => "crates",
        _ => "apps", // fallback
    }
}