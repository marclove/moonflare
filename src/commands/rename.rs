use crate::ui::MoonflareUI;
use crate::utils::fs::is_moonflare_workspace;
use anyhow::{Result, bail};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

pub struct RenameCommand {
    ui: MoonflareUI,
}

impl RenameCommand {
    pub fn new() -> Self {
        Self {
            ui: MoonflareUI::new(),
        }
    }

    pub async fn execute(&self, current_name: &str, new_name: &str) -> Result<()> {
        // Check if we're in a Moonflare workspace
        if !is_moonflare_workspace() {
            bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        self.ui
            .render_header(
                "Renaming project",
                Some(&format!("Renaming '{}' to '{}'", current_name, new_name)),
            )
            .map_err(|e| anyhow::anyhow!("UI render error: {}", e))?;

        // Find the project and determine its type
        let (project_path, project_type) = self.find_project(current_name)?;

        // Check if new name already exists
        let new_project_path = project_path.parent().unwrap().join(new_name);
        if new_project_path.exists() {
            bail!("A project with the name '{}' already exists", new_name);
        }

        // Update wrangler config if it's a web project
        if matches!(project_type.as_str(), "astro" | "react" | "durable-object") {
            self.update_wrangler_config(&project_path, new_name)?;
        }

        // Rename the directory
        fs::rename(&project_path, &new_project_path)?;

        self.ui
            .render_success(&format!(
                "Successfully renamed {} project '{}' to '{}'",
                project_type, current_name, new_name
            ))
            .map_err(|e| anyhow::anyhow!("UI render error: {}", e))?;

        Ok(())
    }

    fn find_project(&self, name: &str) -> Result<(PathBuf, String)> {
        // Check each project type directory
        let project_types = vec![
            ("astro", "sites"),
            ("react", "apps"),
            ("durable-object", "workers"),
            ("crate", "crates"),
        ];

        for (project_type, directory) in project_types {
            let project_path = Path::new(directory).join(name);
            if project_path.exists() {
                return Ok((project_path, project_type.to_string()));
            }
        }

        bail!("Project '{}' not found", name);
    }

    fn update_wrangler_config(&self, project_path: &Path, new_name: &str) -> Result<()> {
        // Check for wrangler.toml first
        let toml_path = project_path.join("wrangler.toml");
        if toml_path.exists() {
            self.update_wrangler_toml(&toml_path, new_name)?;
            return Ok(());
        }

        // Check for wrangler.json
        let json_path = project_path.join("wrangler.json");
        if json_path.exists() {
            self.update_wrangler_json(&json_path, new_name)?;
            return Ok(());
        }

        // Check for wrangler.jsonc
        let jsonc_path = project_path.join("wrangler.jsonc");
        if jsonc_path.exists() {
            self.update_wrangler_jsonc(&jsonc_path, new_name)?;
            return Ok(());
        }

        // No wrangler config found - this might be okay for some projects
        println!("Warning: No wrangler config file found in project directory");
        Ok(())
    }

    fn update_wrangler_toml(&self, toml_path: &Path, new_name: &str) -> Result<()> {
        let content = fs::read_to_string(toml_path)?;
        let mut toml: TomlValue = toml::from_str(&content)?;

        if let Some(table) = toml.as_table_mut() {
            table.insert("name".to_string(), TomlValue::String(new_name.to_string()));
        }

        let updated_content = toml::to_string(&toml)?;
        fs::write(toml_path, updated_content)?;

        println!("Updated wrangler.toml with new project name");
        Ok(())
    }

    fn update_wrangler_json(&self, json_path: &Path, new_name: &str) -> Result<()> {
        let content = fs::read_to_string(json_path)?;
        let mut json: Value = serde_json::from_str(&content)?;

        if let Some(obj) = json.as_object_mut() {
            obj.insert("name".to_string(), Value::String(new_name.to_string()));
        }

        let updated_content = serde_json::to_string_pretty(&json)?;
        fs::write(json_path, updated_content)?;

        println!("Updated wrangler.json with new project name");
        Ok(())
    }

    fn update_wrangler_jsonc(&self, jsonc_path: &Path, new_name: &str) -> Result<()> {
        // For .jsonc files, we need to preserve comments while updating the name
        let content = fs::read_to_string(jsonc_path)?;

        // Simple regex-based replacement to preserve comments
        // This looks for "name": "old_name" and replaces with "name": "new_name"
        let name_pattern = regex::Regex::new(r#""name"\s*:\s*"[^"]*""#)?;
        let replacement = format!(r#""name": "{}""#, new_name);
        let updated_content = name_pattern.replace(&content, replacement.as_str());

        fs::write(jsonc_path, updated_content.as_bytes())?;

        println!("Updated wrangler.jsonc with new project name");
        Ok(())
    }
}
