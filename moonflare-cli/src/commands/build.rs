use anyhow::Result;
use colored::*;
use crate::utils::{moon::run_moon_command, fs::is_moonflare_workspace};

pub struct BuildCommand {}

impl BuildCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, project: Option<&str>) -> Result<()> {
        if !is_moonflare_workspace() {
            anyhow::bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        match project {
            Some(proj) => {
                println!("{}", format!("🔨 Building project '{}'...", proj).cyan().bold());
                run_moon_command(&["run", &format!("{}:build", proj)]).await?;
            },
            None => {
                println!("{}", "🔨 Building all projects...".cyan().bold());
                run_moon_command(&[":build"]).await?;
            }
        }

        println!("✅ {}", "Build completed successfully!".green().bold());
        Ok(())
    }
}