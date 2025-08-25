use anyhow::Result;
use colored::*;
use crate::utils::{cloudflare::deploy_project, fs::is_moonflare_workspace};
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
                println!("{}", format!("🚀 Deploying project '{}'...", proj).cyan().bold());
                
                // Try to find the project in different directories
                let possible_paths = [
                    format!("workers/{}", proj),
                    format!("sites/{}", proj),
                    format!("apps/{}", proj),
                ];
                
                let project_path = possible_paths.iter()
                    .find(|path| Path::new(path).exists())
                    .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", proj))?;
                
                deploy_project(project_path, env).await?;
            },
            None => {
                println!("{}", "🚀 Deploying all deployable projects...".cyan().bold());
                
                // Deploy all projects that have wrangler.toml
                let dirs = ["workers", "sites", "apps"];
                for dir in dirs {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let project_path = entry.path();
                            if project_path.join("wrangler.toml").exists() {
                                if let Some(name) = project_path.file_name() {
                                    println!("{}", format!("Deploying {}...", name.to_string_lossy()).blue());
                                    deploy_project(&project_path.to_string_lossy(), env).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("✅ {}", "Deployment completed successfully!".green().bold());
        Ok(())
    }
}