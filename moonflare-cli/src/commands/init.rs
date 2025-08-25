use anyhow::Result;
use colored::*;
use std::path::Path;
use std::collections::HashMap;
use serde_json::Value;
use crate::templates::{embedded, engine::TemplateEngine};
use crate::utils::{fs::create_directory_if_not_exists, moon::check_moon_installation};

pub struct InitCommand {
    template_engine: TemplateEngine,
}

impl InitCommand {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
        }
    }

    pub async fn execute(&self, name: &str, path: Option<&str>) -> Result<()> {
        println!("{}", "🚀 Initializing new Moonflare monorepo...".cyan().bold());

        // Determine target directory
        let target_dir = match path {
            Some(p) => Path::new(p).join(name),
            None => Path::new(".").join(name),
        };

        // Create directory
        create_directory_if_not_exists(&target_dir)?;
        
        // Check if Moon is installed
        check_moon_installation()?;

        // Prepare template context
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::String(name.to_string()));

        // Generate workspace files
        if let Some(template) = embedded::get_template("workspace") {
            self.template_engine.process_template_files(
                template,
                &target_dir,
                &context
            )?;
        }

        // Create directory structure
        let dirs = ["apps", "sites", "workers", "crates"];
        for dir in dirs {
            create_directory_if_not_exists(&target_dir.join(dir))?;
        }

        println!("✅ {}", format!("Successfully created {} monorepo!", name).green().bold());
        println!();
        println!("{}", "Next steps:".yellow().bold());
        println!("  cd {}", name);
        println!("  moonflare add <type> <name>  # Add a new project");
        println!("  moon setup                  # Initialize Moon workspace");
        println!();
        println!("{}", "Available project types:".blue());
        println!("  • astro          - Astro static site");
        println!("  • react          - React application");
        println!("  • durable-object - Cloudflare Durable Object");
        println!("  • crate          - Rust WASM library");

        Ok(())
    }
}