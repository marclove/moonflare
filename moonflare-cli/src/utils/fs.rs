use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use serde_yaml::Value;

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

/// Check if any crates exist in the workspace
pub fn has_crates() -> bool {
    let crates_dir = Path::new("crates");
    if !crates_dir.exists() {
        return false;
    }
    
    // Check if there are any subdirectories in crates/
    if let Ok(entries) = fs::read_dir(crates_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                return true;
            }
        }
    }
    false
}

/// Get all TypeScript projects (astro, react, durable-object)
pub fn get_typescript_projects() -> Vec<PathBuf> {
    let mut projects = Vec::new();
    
    let directories = ["sites", "apps", "workers"];
    
    for dir in directories {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let moon_yml = entry.path().join("moon.yml");
                    if moon_yml.exists() {
                        projects.push(entry.path());
                    }
                }
            }
        }
    }
    
    projects
}

/// Update a project's moon.yml to add shared-wasm:gather dependency
pub fn add_wasm_dependency_to_project(project_path: &Path) -> Result<()> {
    let moon_yml_path = project_path.join("moon.yml");
    if !moon_yml_path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(&moon_yml_path)?;
    let mut config: Value = serde_yaml::from_str(&content)?;
    
    // Navigate to tasks.build.deps
    if let Some(tasks) = config.get_mut("tasks") {
        if let Some(build_task) = tasks.get_mut("build") {
            if let Some(build_mapping) = build_task.as_mapping_mut() {
                // Get existing deps or create empty array
                let mut deps = build_mapping
                    .get("deps")
                    .and_then(|d| d.as_sequence())
                    .cloned()
                    .unwrap_or_default();
                
                // Check if shared-wasm:gather is already in deps
                let has_wasm_dep = deps.iter().any(|dep| {
                    dep.as_str() == Some("shared-wasm:gather")
                });
                
                if !has_wasm_dep {
                    // Add shared-wasm:gather to deps
                    deps.push(Value::String("shared-wasm:gather".to_string()));
                    build_mapping.insert(
                        Value::String("deps".to_string()),
                        Value::Sequence(deps)
                    );
                }
                
                // Also add WASM inputs if not present
                let mut inputs = build_mapping
                    .get("inputs")
                    .and_then(|i| i.as_sequence())
                    .cloned()
                    .unwrap_or_default();
                
                let wasm_input = "/shared-wasm/*.wasm";
                let has_wasm_input = inputs.iter().any(|input| {
                    input.as_str() == Some(wasm_input)
                });
                
                if !has_wasm_input {
                    inputs.push(Value::String(wasm_input.to_string()));
                    build_mapping.insert(
                        Value::String("inputs".to_string()),
                        Value::Sequence(inputs)
                    );
                }
            }
        }
    }
    
    // Write back to file
    let updated_content = serde_yaml::to_string(&config)?;
    fs::write(&moon_yml_path, updated_content)?;
    
    Ok(())
}

/// Check if a project already has WASM dependency
pub fn has_wasm_dependency(project_path: &Path) -> bool {
    let moon_yml_path = project_path.join("moon.yml");
    if !moon_yml_path.exists() {
        return false;
    }
    
    if let Ok(content) = fs::read_to_string(&moon_yml_path) {
        if let Ok(config) = serde_yaml::from_str::<Value>(&content) {
            if let Some(tasks) = config.get("tasks") {
                if let Some(build_task) = tasks.get("build") {
                    if let Some(deps) = build_task.get("deps") {
                        if let Some(deps_array) = deps.as_sequence() {
                            return deps_array.iter().any(|dep| {
                                dep.as_str() == Some("shared-wasm:gather")
                            });
                        }
                    }
                }
            }
        }
    }
    
    false
}

/// Add a crate's build task as a dependency to shared-wasm:gather
pub fn add_crate_build_dependency_to_shared_wasm(crate_name: &str) -> Result<()> {
    let shared_wasm_moon_yml = Path::new("shared-wasm/moon.yml");
    if !shared_wasm_moon_yml.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(shared_wasm_moon_yml)?;
    let mut config: Value = serde_yaml::from_str(&content)?;
    
    // Navigate to tasks.gather.deps
    if let Some(tasks) = config.get_mut("tasks") {
        if let Some(gather_task) = tasks.get_mut("gather") {
            if let Some(gather_mapping) = gather_task.as_mapping_mut() {
                // Get existing deps or create empty array
                let mut deps = gather_mapping
                    .get("deps")
                    .and_then(|d| d.as_sequence())
                    .cloned()
                    .unwrap_or_default();
                
                // Create the crate build target
                let crate_target = format!("{}:build", crate_name);
                
                // Check if this crate build is already in deps
                let has_crate_dep = deps.iter().any(|dep| {
                    dep.as_str() == Some(&crate_target)
                });
                
                if !has_crate_dep {
                    // Add crate:build to deps
                    deps.push(Value::String(crate_target));
                    gather_mapping.insert(
                        Value::String("deps".to_string()),
                        Value::Sequence(deps)
                    );
                }
            }
        }
    }
    
    // Write back to file
    let updated_content = serde_yaml::to_string(&config)?;
    fs::write(shared_wasm_moon_yml, updated_content)?;
    
    Ok(())
}