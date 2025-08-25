use crate::utils::{fs::is_moonflare_workspace, moon::run_moon_command};
use anyhow::Result;
use colored::*;

pub struct DevCommand {}

impl DevCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, project: Option<&str>) -> Result<()> {
        if !is_moonflare_workspace() {
            anyhow::bail!("Not in a Moonflare workspace. Run 'moonflare init <name>' first.");
        }

        match project {
            Some(proj) => {
                println!(
                    "{}",
                    format!("Starting development server for '{}'...", proj)
                        .cyan()
                        .bold()
                );
                run_moon_command(&["run", &format!("{}:dev", proj)]).await?;
            }
            None => {
                println!("{}", "Starting all development servers...".cyan().bold());
                run_moon_command(&[":dev"]).await?;
            }
        }

        Ok(())
    }
}
