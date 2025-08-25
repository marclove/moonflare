use crate::templates::{embedded, engine::TemplateEngine};
use crate::utils::fs::{
    add_crate_build_dependency_to_shared_wasm, add_wasm_dependency_to_project,
    create_directory_if_not_exists, get_project_directory, get_typescript_projects, has_crates,
    has_wasm_dependency, is_moonflare_workspace,
};
use crate::ui::MoonflareUI;
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct AddCommand {
    template_engine: TemplateEngine,
    ui: MoonflareUI,
}

impl AddCommand {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            ui: MoonflareUI::new(),
        }
    }

    pub async fn execute(&self, project_type: &str, name: &str) -> Result<()> {
        // Check if we're in a Moonflare workspace
        if !is_moonflare_workspace() {
            bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        self.ui.render_header(
            "Adding project", 
            Some(&format!("Creating {} project '{}'", project_type, name))
        ).map_err(|e| anyhow::anyhow!("UI render error: {}", e))?;

        // Get the appropriate directory for this project type
        let project_dir = get_project_directory(project_type);
        let target_path = Path::new(project_dir).join(name);

        // Create project directory
        create_directory_if_not_exists(&target_path)?;

        // Get template content
        let template = embedded::get_template(project_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown project type: {}", project_type))?;

        // Prepare template context
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::String(name.to_string()));

        // Add additional context based on project type
        if project_type == "durable-object" {
            context.insert("name_upper".to_string(), Value::String(name.to_uppercase()));
            context.insert("name_title".to_string(), Value::String(to_title_case(name)));
        }

        // For TypeScript projects, check if we need WASM dependencies
        let is_typescript_project = matches!(project_type, "astro" | "react" | "durable-object");
        let should_add_wasm_deps = is_typescript_project && has_crates();

        // Add WASM context if needed
        if should_add_wasm_deps {
            context.insert("has_wasm".to_string(), Value::Bool(true));
        }

        // Generate project files
        self.template_engine
            .process_template_files(template, &target_path, &context)?;

        // Handle special post-generation tasks
        match project_type {
            "crate" => {
                // When adding a crate, update all existing TypeScript projects to depend on WASM
                self.add_wasm_dependencies_to_existing_projects().await?;
                // Update shared-wasm to depend on this new crate
                self.add_crate_dependency_to_shared_wasm(name).await?;
            }
            "react" | "durable-object" => {
                // Generate Wrangler types for TypeScript support
                self.generate_wrangler_types(&target_path).await?;
            }
            "astro" => {
                // WASM dependencies are handled by template context
            }
            _ => {}
        }

        self.ui.render_success(&format!(
            "Successfully created {} project '{}'", 
            project_type, 
            name
        )).map_err(|e| anyhow::anyhow!("UI render error: {}", e))?;
        
        self.ui.render_next_steps_for_project(name, project_type)
            .map_err(|e| anyhow::anyhow!("UI render error: {}", e))?;

        Ok(())
    }

    async fn add_wasm_dependencies_to_existing_projects(&self) -> Result<()> {
        let typescript_projects = get_typescript_projects();
        let mut updated_count = 0;

        for project_path in typescript_projects {
            if !has_wasm_dependency(&project_path) {
                add_wasm_dependency_to_project(&project_path)?;
                updated_count += 1;
            }
        }

        if updated_count > 0 {
            if let Ok(ui) = MoonflareUI::new().render_success(&format!(
                "Updated {} existing TypeScript project(s) to use WASM",
                updated_count
            )) {
                ui
            } else {
                println!("Updated {} existing TypeScript project(s) to use WASM", updated_count);
            }
        }

        Ok(())
    }

    async fn add_crate_dependency_to_shared_wasm(&self, crate_name: &str) -> Result<()> {
        add_crate_build_dependency_to_shared_wasm(crate_name)?;
        Ok(())
    }

    async fn generate_wrangler_types(&self, project_path: &Path) -> Result<()> {
        use std::process::Command;
        use which::which;

        // Check if wrangler is available
        if which("wrangler").is_err() {
            // Don't fail if wrangler isn't installed, just warn
            println!("Warning: Wrangler CLI not found. Install with: npm install -g wrangler");
            println!("TypeScript definitions will be generated when building the project.");
            return Ok(());
        }

        // Run wrangler types in the project directory
        let output = Command::new("wrangler")
            .args(&["types"])
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("Generated TypeScript definitions for Cloudflare Workers");
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("Warning: Failed to generate Wrangler types: {}", stderr);
                    println!("TypeScript definitions will be generated when building the project.");
                }
            }
            Err(_) => {
                println!("Warning: Could not run wrangler types command");
                println!("TypeScript definitions will be generated when building the project.");
            }
        }

        Ok(())
    }
}

fn to_title_case(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 || s.chars().nth(i - 1) == Some('_') || s.chars().nth(i - 1) == Some('-') {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_lowercase().collect::<String>()
            }
        })
        .collect()
}
