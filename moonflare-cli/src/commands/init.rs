use miette::{Result, IntoDiagnostic};
use colored::*;
use std::path::Path;
use std::collections::HashMap;
use serde_json::Value;
use crate::templates::{embedded, engine::TemplateEngine};
use crate::utils::{fs::create_directory_if_not_exists, moon::{check_moon_installation, moon_setup}};
use crate::errors::{MoonflareError, validate_workspace_name};

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

        // Validate workspace name
        validate_workspace_name(name).into_diagnostic()?;

        // Determine target directory
        let target_dir = match path {
            Some(p) => Path::new(p).join(name),
            None => Path::new(".").join(name),
        };

        // Check if directory already exists and has content
        if target_dir.exists() {
            if target_dir.is_dir() {
                let is_empty = std::fs::read_dir(&target_dir)
                    .map_err(|e| MoonflareError::permission_denied(target_dir.clone(), e))
                    .into_diagnostic()?
                    .next()
                    .is_none();
                
                if !is_empty {
                    return Err(MoonflareError::workspace_directory_exists(target_dir)).into_diagnostic();
                }
            } else {
                // Path exists but is not a directory
                return Err(MoonflareError::workspace_directory_exists(target_dir)).into_diagnostic();
            }
        }

        // Create directory with better error handling
        create_directory_if_not_exists(&target_dir)
            .map_err(|e| {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            MoonflareError::permission_denied(target_dir.clone(), std::io::Error::new(io_err.kind(), format!("{}", io_err)))
                        }
                        _ => MoonflareError::file_system_error("create directory", target_dir.clone(), std::io::Error::new(io_err.kind(), format!("{}", io_err)))
                    }
                } else {
                    MoonflareError::file_system_error("create directory", target_dir.clone(), 
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                }
            })
            .into_diagnostic()?;
        
        // Check if Moon is installed
        check_moon_installation()
            .map_err(|e| MoonflareError::moon_not_found(Some(e.to_string())))
            .into_diagnostic()?;

        // Prepare template context
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::String(name.to_string()));

        // Generate workspace files
        if let Some(template) = embedded::get_template("workspace") {
            self.template_engine.process_template_files(
                template,
                &target_dir,
                &context
            ).map_err(|e| MoonflareError::template_error("workspace", Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))))
            .into_diagnostic()?;
        }

        // Create directory structure
        let dirs = ["apps", "sites", "workers", "crates"];
        for dir in dirs {
            create_directory_if_not_exists(&target_dir.join(dir))
                .map_err(|e| {
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        MoonflareError::file_system_error(
                            &format!("create {} directory", dir), 
                            target_dir.join(dir), 
                            std::io::Error::new(io_err.kind(), format!("{}", io_err))
                        )
                    } else {
                        MoonflareError::file_system_error(
                            &format!("create {} directory", dir), 
                            target_dir.join(dir), 
                            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                        )
                    }
                })
                .into_diagnostic()?;
        }

        // Run moon setup in the new workspace
        println!("{}", "🔧 Initializing Moon workspace...".blue());
        let current_dir = std::env::current_dir()
            .map_err(|e| MoonflareError::file_system_error("get current directory", std::env::current_dir().unwrap_or_default(), e))
            .into_diagnostic()?;
        
        std::env::set_current_dir(&target_dir)
            .map_err(|e| MoonflareError::file_system_error("change directory", target_dir.clone(), e))
            .into_diagnostic()?;
        
        match moon_setup().await {
            Ok(_) => println!("✅ {}", "Moon workspace initialized".green()),
            Err(e) => {
                // Restore directory before potentially returning error
                let _ = std::env::set_current_dir(&current_dir);
                
                // For now, just warn about Moon setup failure rather than failing entirely
                println!("⚠️  {}", format!("Moon setup failed: {}", e).yellow());
                println!("You can run 'moon setup' manually later.");
                
                // Uncomment this line if you want Moon setup failure to be fatal:
                // return Err(MoonflareError::moon_setup_failed(target_dir, Box::new(e), None));
            }
        }
        
        // Restore original directory
        std::env::set_current_dir(&current_dir)
            .map_err(|e| MoonflareError::file_system_error("restore directory", current_dir.clone(), e))
            .into_diagnostic()?;

        println!("✅ {}", format!("Successfully created {} monorepo!", name).green().bold());
        println!();
        println!("{}", "Next steps:".yellow().bold());
        println!("  cd {}", name);
        println!("  moonflare add <type> <name>  # Add a new project");
        println!();
        println!("{}", "Available project types:".blue());
        println!("  • astro          - Astro static site");
        println!("  • react          - React application");
        println!("  • durable-object - Cloudflare Durable Object");
        println!("  • crate          - Rust WASM library");

        Ok(())
    }
}