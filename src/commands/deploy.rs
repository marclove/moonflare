use crate::utils::{fs::is_moonflare_workspace, moon::run_moon_command};
use anyhow::Result;
use colored::*;
use std::path::Path;

pub struct DeployCommand {}

impl DeployCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, project: Option<&str>, env: Option<&str>) -> Result<()> {
        if !is_moonflare_workspace() {
            anyhow::bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        match project {
            Some(proj) => {
                println!(
                    "{}",
                    format!("Deploying project '{}'...", proj).cyan().bold()
                );

                // Try to find the project in different directories
                let possible_paths = [
                    format!("workers/{}", proj),
                    format!("sites/{}", proj),
                    format!("apps/{}", proj),
                ];

                let _project_path = possible_paths
                    .iter()
                    .find(|path| Path::new(path).exists())
                    .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", proj))?;

                // Use Moon to run the deploy task, which handles project-level dependencies
                let moon_target = format!("{}:deploy", proj);
                if let Some(environment) = env {
                    println!(
                        "{}",
                        format!("Deploying to environment: {}", environment).yellow()
                    );
                }

                // Moon will inherit environment variables from the current process
                // We can use the moon command directly with environment variables
                if let Some(environment) = env {
                    let cmd_args = vec!["run", &moon_target];
                    // We'll set the WRANGLER_ENV environment variable for this execution
                    std::process::Command::new("moon")
                        .args(&cmd_args)
                        .env("WRANGLER_ENV", environment)
                        .status()?;
                } else {
                    run_moon_command(&["run", &moon_target]).await?;
                }
            }
            None => {
                println!("{}", "Deploying all deployable projects...".cyan().bold());

                // Deploy all projects that have Wrangler configuration files (wrangler.toml, wrangler.json, or wrangler.jsonc)
                let dirs = ["workers", "sites", "apps"];
                for dir in dirs {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let project_path = entry.path();
                            // Check for any Wrangler configuration file format
                            if (project_path.join("wrangler.toml").exists()
                                || project_path.join("wrangler.json").exists()
                                || project_path.join("wrangler.jsonc").exists())
                                && let Some(name) = project_path.file_name()
                            {
                                println!(
                                    "{}",
                                    format!("Deploying {}...", name.to_string_lossy()).blue()
                                );
                                // Use Moon to run the deploy task for each project
                                let moon_target = format!("{}:deploy", name.to_string_lossy());

                                // Moon will inherit environment variables from the current process
                                if let Some(environment) = env {
                                    let cmd_args = vec!["run", &moon_target];
                                    std::process::Command::new("moon")
                                        .args(&cmd_args)
                                        .env("WRANGLER_ENV", environment)
                                        .status()?;
                                } else {
                                    run_moon_command(&["run", &moon_target]).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("{}", "Deployment completed successfully!".green().bold());
        Ok(())
    }
}
