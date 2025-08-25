use clap::{Parser, Subcommand};
use miette::Result;

mod commands;
mod templates;
mod utils;
mod errors;

use commands::{init::InitCommand, add::AddCommand, build::BuildCommand, dev::DevCommand, deploy::DeployCommand};

#[derive(Parser)]
#[command(
    name = "moonflare",
    about = "A CLI utility for managing Cloudflare-focused monorepos with Moon",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize a new Cloudflare monorepo")]
    Init {
        #[arg(help = "Name of the monorepo")]
        name: String,
        #[arg(long, help = "Directory to create the monorepo in")]
        path: Option<String>,
        #[arg(long, help = "Force initialization in non-empty directories")]
        force: bool,
    },
    
    #[command(about = "Add a new project to the monorepo")]
    Add {
        #[arg(help = "Type of project (astro, react, worker, durable-object, crate)")]
        project_type: String,
        #[arg(help = "Name of the project")]
        name: String,
    },
    
    #[command(about = "Build project(s)")]
    Build {
        #[arg(help = "Specific project to build (optional)")]
        project: Option<String>,
    },
    
    #[command(about = "Start development server")]
    Dev {
        #[arg(help = "Specific project to run (optional)")]
        project: Option<String>,
    },
    
    #[command(about = "Deploy project(s) to Cloudflare")]
    Deploy {
        #[arg(help = "Specific project to deploy (optional)")]
        project: Option<String>,
        #[arg(long, help = "Environment to deploy to")]
        env: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install miette panic and error hooks for better error reporting
    miette::set_panic_hook();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name, path, force } => {
            let init_cmd = InitCommand::new();
            init_cmd.execute(&name, path.as_deref(), force).await?;
        },
        Commands::Add { project_type, name } => {
            let add_cmd = AddCommand::new();
            add_cmd.execute(&project_type, &name).await
                .map_err(|e| miette::miette!("Add command failed: {}", e))?;
        },
        Commands::Build { project } => {
            let build_cmd = BuildCommand::new();
            build_cmd.execute(project.as_deref()).await?;
        },
        Commands::Dev { project } => {
            let dev_cmd = DevCommand::new();
            dev_cmd.execute(project.as_deref()).await
                .map_err(|e| miette::miette!("Dev command failed: {}", e))?;
        },
        Commands::Deploy { project, env } => {
            let deploy_cmd = DeployCommand::new();
            deploy_cmd.execute(project.as_deref(), env.as_deref()).await
                .map_err(|e| miette::miette!("Deploy command failed: {}", e))?;
        },
    }
    
    Ok(())
}
