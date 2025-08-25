use anyhow::{Result, bail};
use which::which;
use std::process::Command;
use colored::*;

pub fn check_wrangler_installation() -> Result<()> {
    match which("wrangler") {
        Ok(_) => {
            println!("{}", "Wrangler CLI is installed".green());
            Ok(())
        },
        Err(_) => {
            println!("{}", "Wrangler CLI not found".yellow());
            println!("Please install Wrangler:");
            println!("  npm install -g wrangler");
            bail!("Wrangler CLI installation required");
        }
    }
}

pub async fn deploy_project(project_path: &str, env: Option<&str>) -> Result<()> {
    check_wrangler_installation()?;
    
    let mut args = vec!["deploy"];
    if let Some(environment) = env {
        args.push("--env");
        args.push(environment);
    }
    
    let output = Command::new("wrangler")
        .current_dir(project_path)
        .args(&args)
        .output()?;
    
    if output.status.success() {
        println!("{}", "Deployment successful!".green());
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Deployment failed: {}", error);
    }
}