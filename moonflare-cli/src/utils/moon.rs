use anyhow::{Result, bail};
use which::which;
use std::process::Command;
use colored::*;
use serde::{Deserialize, Serialize};

pub fn check_moon_installation() -> Result<()> {
    match which("moon") {
        Ok(_) => {
            println!("✅ {}", "Moon CLI is installed".green());
            Ok(())
        },
        Err(_) => {
            println!("⚠️  {}", "Moon CLI not found".yellow());
            println!("Installing Moon via proto...");
            
            // Try to install via proto
            let output = Command::new("proto")
                .args(&["install", "moon"])
                .output();
                
            match output {
                Ok(result) if result.status.success() => {
                    println!("✅ {}", "Moon CLI installed successfully".green());
                    Ok(())
                },
                _ => {
                    eprintln!("❌ {}", "Failed to install Moon CLI".red());
                    eprintln!("Please install Moon manually:");
                    eprintln!("  curl -fsSL https://moonrepo.dev/install/moon.sh | bash");
                    bail!("Moon CLI installation required");
                }
            }
        }
    }
}

pub async fn run_moon_command(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("moon");
    cmd.args(args);
    
    let output = cmd.output()?;
    
    if output.status.success() {
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Moon command failed: {}", error);
    }
}

pub async fn moon_setup() -> Result<()> {
    println!("{}", "Setting up Moon workspace...".blue());
    run_moon_command(&["setup"]).await
}

// Run a Moon command and return the output without printing it
pub async fn run_moon_command_silent(args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("moon");
    cmd.args(args);
    
    let output = cmd.output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Moon command failed: {}", error);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoonProject {
    pub id: String,
    pub source: String,
    pub language: String,
    pub layer: Option<String>,
    pub stack: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoonProjectsResponse {
    pub projects: Vec<MoonProject>,
}

// Query all available projects in the Moon workspace
pub async fn query_projects() -> Result<Vec<MoonProject>> {
    let output = run_moon_command_silent(&["query", "projects", "--json"]).await?;
    
    // Parse the JSON output
    let response: MoonProjectsResponse = serde_json::from_str(&output)
        .map_err(|e| anyhow::anyhow!("Failed to parse Moon projects response: {}", e))?;
    
    Ok(response.projects)
}

// Check if a specific project exists and return available project names
pub async fn validate_project_exists(project_name: &str) -> Result<Option<Vec<String>>> {
    match query_projects().await {
        Ok(projects) => {
            // Check if the project exists by ID
            let project_exists = projects.iter().any(|p| p.id == project_name);
            
            if project_exists {
                Ok(None) // Project exists, no error
            } else {
                // Project doesn't exist, return available project names with type info
                let available_projects: Vec<String> = projects
                    .iter()
                    .map(|p| {
                        let stack_info = match p.stack.as_deref() {
                            Some("frontend") => " (frontend)",
                            Some("backend") => " (backend)", 
                            _ => ""
                        };
                        format!("{}{}", p.id, stack_info)
                    })
                    .collect();
                Ok(Some(available_projects))
            }
        }
        Err(e) => {
            // If we can't query projects, fall back to the original behavior
            bail!("Failed to query available projects: {}", e);
        }
    }
}