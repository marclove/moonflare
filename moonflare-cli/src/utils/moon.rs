use anyhow::{Result, bail};
use which::which;
use std::process::Command;
use colored::*;

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