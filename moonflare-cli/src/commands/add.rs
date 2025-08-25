use anyhow::{Result, bail};
use colored::*;
use std::path::Path;
use std::collections::HashMap;
use serde_json::Value;
use crate::templates::{embedded, engine::TemplateEngine};
use crate::utils::fs::{create_directory_if_not_exists, is_moonflare_workspace, get_project_directory};

pub struct AddCommand {
    template_engine: TemplateEngine,
}

impl AddCommand {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
        }
    }

    pub async fn execute(&self, project_type: &str, name: &str) -> Result<()> {
        // Check if we're in a Moonflare workspace
        if !is_moonflare_workspace() {
            bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        println!("{}", format!("🔨 Adding {} project '{}'...", project_type, name).cyan().bold());

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
        match project_type {
            "durable-object" => {
                context.insert("name_upper".to_string(), Value::String(name.to_uppercase()));
                context.insert("name_title".to_string(), Value::String(to_title_case(name)));
            },
            _ => {}
        }

        // Generate project files
        self.template_engine.process_template_files(
            template,
            &target_path,
            &context
        )?;

        // Handle special post-generation tasks
        match project_type {
            "crate" => {
                self.update_shared_wasm_config(name).await?;
            },
            _ => {}
        }

        println!("✅ {}", format!("Successfully created {} project '{}'", project_type, name).green().bold());
        println!();
        println!("{}", "Next steps:".yellow().bold());
        match project_type {
            "astro" => {
                println!("  cd {}/{}", project_dir, name);
                println!("  pnpm install");
                println!("  pnpm dev");
            },
            "react" => {
                println!("  cd {}/{}", project_dir, name);
                println!("  pnpm install");
                println!("  pnpm dev");
            },
            "durable-object" => {
                println!("  cd {}/{}", project_dir, name);
                println!("  pnpm install");
                println!("  pnpm dev");
            },
            "crate" => {
                println!("  cd {}/{}", project_dir, name);
                println!("  cargo build --target wasm32-unknown-unknown --release");
            },
            _ => {}
        }

        Ok(())
    }

    async fn update_shared_wasm_config(&self, crate_name: &str) -> Result<()> {
        // TODO: Update shared-wasm/moon.yml to add the new crate as a dependency
        // For now, just print a note
        println!("{}", "📝 Note: Add this crate to shared-wasm/moon.yml dependencies manually".yellow());
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