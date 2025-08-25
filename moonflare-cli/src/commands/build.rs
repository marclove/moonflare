use miette::{Result, IntoDiagnostic};
use colored::*;
use std::env;
use crate::utils::{moon::{run_moon_command_with_error, validate_project_exists}, fs::is_moonflare_workspace};
use crate::errors::MoonflareError;

pub struct BuildCommand {}

impl BuildCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, project: Option<&str>) -> Result<()> {
        if !is_moonflare_workspace() {
            let current_dir = env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            return Err(MoonflareError::not_in_workspace(current_dir, vec![])).into_diagnostic();
        }

        match project {
            Some(proj) => {
                println!("{}", format!("Building project '{}'...", proj).cyan().bold());
                
                // Check if the project exists by querying Moon for available projects
                match validate_project_exists(proj).await {
                    Ok(Some(available_projects)) => {
                        // Project doesn't exist, show helpful error with available projects
                        let current_dir = env::current_dir()
                            .unwrap_or_else(|_| std::path::PathBuf::from("."));
                        
                        let available_list = if available_projects.is_empty() {
                            "No projects found in this workspace. Add projects with 'moonflare add <type> <name>'.".to_string()
                        } else {
                            format!("Available projects:\n  • {}", available_projects.join("\n  • "))
                        };
                        
                        return Err(MoonflareError::project_not_found(
                            proj, 
                            current_dir, 
                            Some(available_list)
                        )).into_diagnostic();
                    }
                    Ok(None) => {
                        // Project exists, proceed with build
                    }
                    Err(_) => {
                        // Failed to query projects, fall back to original behavior
                        // This allows the command to work even if Moon query fails
                    }
                }
                
                // Run the actual build command with structured error handling
                if let Err(moon_error) = run_moon_command_with_error(&["run", &format!("{}:build", proj)]).await {
                    return Err(moon_error).into_diagnostic();
                }
            },
            None => {
                println!("{}", "Building all projects...".cyan().bold());
                
                // Run build all with structured error handling
                if let Err(moon_error) = run_moon_command_with_error(&[":build"]).await {
                    return Err(moon_error).into_diagnostic();
                }
            }
        }

        println!("{}", "Build completed successfully!".green().bold());
        Ok(())
    }
}