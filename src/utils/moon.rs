use crate::errors::MoonflareError;
use anyhow::{Result, bail};
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use which::which;

// Helper function to find Moon CLI in known locations
fn find_moon_binary() -> Option<PathBuf> {
    let is_ci = std::env::var("CI").unwrap_or_default().to_lowercase() == "true"
        || std::env::var("GITHUB_ACTIONS")
            .unwrap_or_default()
            .to_lowercase()
            == "true";

    if is_ci {
        eprintln!("=== MOON BINARY SEARCH IN CI ===");
        eprintln!(
            "Environment: CI={}, GITHUB_ACTIONS={}",
            std::env::var("CI").unwrap_or_default(),
            std::env::var("GITHUB_ACTIONS").unwrap_or_default()
        );
        eprintln!(
            "Current PATH: {}",
            std::env::var("PATH").unwrap_or_default()
        );
        eprintln!(
            "HOME directory: {}",
            std::env::var("HOME").unwrap_or_default()
        );
        eprintln!();
    }

    // First try the normal PATH lookup
    if let Ok(path) = which("moon") {
        if is_ci {
            eprintln!("✓ Found Moon via PATH: {}", path.display());
            if let Ok(metadata) = std::fs::metadata(&path) {
                eprintln!(
                    "  File exists: {}, Size: {} bytes",
                    metadata.is_file(),
                    metadata.len()
                );
            }
        }
        return Some(path);
    } else if is_ci {
        eprintln!("✗ Moon not found in PATH via which()");
    }

    if is_ci {
        eprintln!();
        eprintln!("Checking expected installation locations:");

        // Check /usr/local/bin/moon (where we install it in CI)
        let usr_local_moon = PathBuf::from("/usr/local/bin/moon");
        eprintln!(
            "  /usr/local/bin/moon: {}",
            if usr_local_moon.exists() {
                format!(
                    "EXISTS ({})",
                    if usr_local_moon.is_file() {
                        "file"
                    } else {
                        "not a file"
                    }
                )
            } else {
                "NOT FOUND".to_string()
            }
        );

        if usr_local_moon.exists() && usr_local_moon.is_file() {
            eprintln!("✓ Using Moon from /usr/local/bin/moon");
            return Some(usr_local_moon);
        }

        // Check ~/.moon/bin/moon (default Moon installation location)
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/runner".to_string());
        let home_moon = PathBuf::from(format!("{}/.moon/bin/moon", home));
        eprintln!(
            "  {}/.moon/bin/moon: {}",
            home,
            if home_moon.exists() {
                format!(
                    "EXISTS ({})",
                    if home_moon.is_file() {
                        "file"
                    } else {
                        "not a file"
                    }
                )
            } else {
                "NOT FOUND".to_string()
            }
        );

        if home_moon.exists() && home_moon.is_file() {
            eprintln!("✓ Using Moon from {}/.moon/bin/moon", home);
            return Some(home_moon);
        }

        // Check common proto installation paths in CI for debugging
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/runner".to_string());
        let proto_paths = [
            format!("{}/.proto/shims/moon", home),
            format!("{}/.proto/bin/moon", home),
            "/home/runner/.proto/shims/moon".to_string(),
            "/home/runner/.proto/bin/moon".to_string(),
        ];

        eprintln!("Legacy proto paths (for debugging):");
        for path_str in &proto_paths {
            let path = PathBuf::from(path_str);
            eprintln!(
                "  {}: {}",
                path_str,
                if path.exists() {
                    format!(
                        "EXISTS ({})",
                        if path.is_file() { "file" } else { "not a file" }
                    )
                } else {
                    "NOT FOUND".to_string()
                }
            );
        }

        eprintln!();
        eprintln!("=== MOON BINARY SEARCH FAILED ===");
    }

    None
}

pub fn check_moon_installation() -> Result<()> {
    let is_ci = std::env::var("CI").unwrap_or_default().to_lowercase() == "true"
        || std::env::var("GITHUB_ACTIONS")
            .unwrap_or_default()
            .to_lowercase()
            == "true";

    match find_moon_binary() {
        Some(_) => {
            println!("{}", "Moon CLI is installed".green());
            Ok(())
        }
        None => {
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
    let moon_binary = find_moon_binary().unwrap_or_else(|| PathBuf::from("moon"));

    let mut cmd = Command::new(moon_binary);
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
    let moon_binary = find_moon_binary().unwrap_or_else(|| PathBuf::from("moon"));

    let mut cmd = Command::new(moon_binary);
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
        match find_moon_binary() {
            Some(_) => Ok(()),
            None => {
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
    let moon_binary = find_moon_binary().unwrap_or_else(|| PathBuf::from("moon"));

    let mut cmd = Command::new(moon_binary);
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
