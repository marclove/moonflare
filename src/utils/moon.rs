use crate::errors::MoonflareError;
use anyhow::{Result, bail};
use colored::*;
use serde::{Deserialize, Serialize};
use std::process::Command;
use which::which;

pub fn check_moon_installation() -> Result<()> {
    let is_ci = std::env::var("CI").unwrap_or_default().to_lowercase() == "true"
        || std::env::var("GITHUB_ACTIONS")
            .unwrap_or_default()
            .to_lowercase()
            == "true";

    match which("moon") {
        Ok(_) => {
            println!("{}", "Moon CLI is installed".green());
            Ok(())
        }
        Err(_) => {
            if is_ci {
                // In CI, provide specific guidance about setup-toolchain action
                eprintln!("{}", "Moon CLI not found in CI environment".red());
                eprintln!(
                    "This usually means the moonrepo/setup-toolchain action is missing or misconfigured."
                );
                eprintln!();
                eprintln!("To fix this, ensure your GitHub Actions workflow includes:");
                eprintln!();
                eprintln!("  - name: Setup Moon toolchain");
                eprintln!("    uses: moonrepo/setup-toolchain@v0");
                eprintln!("    with:");
                eprintln!("      auto-install: true");
                eprintln!("      cache: true");
                eprintln!();
                eprintln!("For more information, see: https://github.com/moonrepo/setup-toolchain");
                bail!("Moon CLI not available in CI");
            } else {
                // Local development - try to install via proto
                println!("{}", "Moon CLI not found".yellow());
                println!("Installing Moon via proto...");

                let output = Command::new("proto").args(["install", "moon"]).output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("{}", "Moon CLI installed successfully".green());
                        Ok(())
                    }
                    _ => {
                        eprintln!("{}", "Failed to install Moon CLI".red());
                        eprintln!("Please install Moon manually:");
                        eprintln!("  curl -fsSL https://moonrepo.dev/install/moon.sh | bash");
                        bail!("Moon CLI installation required");
                    }
                }
            }
        }
    }
}

pub async fn run_moon_command(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("moon");
    cmd.args(args);

    let status = cmd.status()?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "Moon command '{}' failed with exit code: {:?}",
            args.join(" "),
            status.code()
        );
    }
}

// Run a Moon command with direct stdio passthrough for best UX
pub async fn run_moon_command_with_error(args: &[&str]) -> std::result::Result<(), MoonflareError> {
    let mut cmd = Command::new("moon");
    cmd.args(args);

    // Let Moon's stdout and stderr pass through directly to preserve colors and formatting
    let status = cmd.status().map_err(|e| {
        MoonflareError::moon_command_failed(
            &args.join(" "),
            &format!("Failed to execute moon command: {}", e),
            None,
        )
    })?;

    if status.success() {
        Ok(())
    } else {
        // Moon has already printed its error to stderr with full formatting
        // Just return a simple error that indicates the command failed
        Err(MoonflareError::moon_command_failed(
            &args.join(" "),
            "", // Empty stderr since Moon already printed to user
            status.code(),
        ))
    }
}

pub async fn moon_setup() -> Result<()> {
    // Check if we're in a CI environment where toolchain is already set up
    let is_ci = std::env::var("CI").unwrap_or_default().to_lowercase() == "true"
        || std::env::var("GITHUB_ACTIONS")
            .unwrap_or_default()
            .to_lowercase()
            == "true";

    if is_ci {
        // In CI, skip moon setup since moonrepo/setup-toolchain action already handles this
        println!("{}", "Skipping Moon setup in CI environment (toolchain already configured by setup-toolchain action)".blue());

        // Verify moon is actually available in CI and provide helpful error if not
        match which::which("moon") {
            Ok(_) => Ok(()),
            Err(_) => {
                bail!(
                    "Moon CLI not found in CI environment. This usually means the moonrepo/setup-toolchain action is missing or misconfigured.\n\n\
                    To fix this, ensure your GitHub Actions workflow includes:\n\n\
                    - name: Setup Moon toolchain\n      \
                    uses: moonrepo/setup-toolchain@v0\n      \
                    with:\n        \
                    auto-install: true\n        \
                    cache: true\n\n\
                    For more information, see: https://github.com/moonrepo/setup-toolchain"
                );
            }
        }
    } else {
        println!("{}", "Setting up Moon workspace...".blue());
        run_moon_command(&["setup"]).await
    }
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
                            _ => "",
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
