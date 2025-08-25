use miette::{Result, IntoDiagnostic};
use std::path::Path;
use std::collections::HashMap;
use serde_json::Value;
use crate::templates::{embedded, engine::TemplateEngine};
use crate::utils::{fs::create_directory_if_not_exists, moon::{check_moon_installation, moon_setup}};
use crate::errors::{MoonflareError, validate_workspace_name};
use crate::ui::MoonflareUI;

pub struct InitCommand {
    template_engine: TemplateEngine,
    ui: MoonflareUI,
}

impl InitCommand {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            ui: MoonflareUI::new(),
        }
    }

    pub async fn execute(&self, name: &str, path: Option<&str>, force: bool) -> Result<()> {
        self.ui.render_header(
            "Moonflare: Supersonic Cloudflare monorepo", 
            Some("Initializing new workspace with Moon build system")
        ).map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;

        // Determine target directory and workspace name
        let (target_dir, workspace_name) = if name == "." {
            // Initialize in current directory
            let current_dir = std::env::current_dir()
                .map_err(|e| MoonflareError::file_system_error("get current directory", std::env::current_dir().unwrap_or_default(), e))
                .into_diagnostic()?;
            
            // Use the directory name as the workspace name
            let dir_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| MoonflareError::invalid_workspace_name(".", vec!["my-workspace".to_string(), "my-app".to_string()]))
                .into_diagnostic()?
                .to_string();
            
            // Validate the directory name as workspace name
            validate_workspace_name(&dir_name).into_diagnostic()?;
            
            (current_dir, dir_name)
        } else {
            // Validate provided workspace name
            validate_workspace_name(name).into_diagnostic()?;
            
            // Determine target directory
            let target_dir = match path {
                Some(p) => Path::new(p).join(name),
                None => Path::new(".").join(name),
            };
            
            (target_dir, name.to_string())
        };

        // Check if directory already exists and has content
        if target_dir.exists() {
            if target_dir.is_dir() {
                let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&target_dir)
                    .map_err(|e| MoonflareError::permission_denied(target_dir.clone(), e))
                    .into_diagnostic()?
                    .collect::<Result<Vec<_>, std::io::Error>>()
                    .map_err(|e| MoonflareError::permission_denied(target_dir.clone(), e))
                    .into_diagnostic()?;
                
                if !entries.is_empty() && !force {
                    // Show beautiful error UI with --force suggestion
                    let dir_name = target_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("directory");
                    
                    let sample_files = entries.iter()
                        .take(3)
                        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    let message = if entries.len() == 1 {
                        format!("Directory is not empty. Directory '{}' contains 1 file: {}", dir_name, sample_files)
                    } else if entries.len() <= 3 {
                        format!("Directory is not empty. Directory '{}' contains {} files: {}", dir_name, entries.len(), sample_files)
                    } else {
                        format!("Directory is not empty. Directory '{}' contains {} files including: {}", dir_name, entries.len(), sample_files)
                    };
                    
                    let init_command = if name == "." {
                        "moonflare init . --force"
                    } else {
                        &format!("moonflare init {} --force", name)
                    };
                    
                    let suggestions = vec![
                        init_command,
                        "Choose a different directory name",
                        "Remove existing files first"
                    ];
                    
                    self.ui.render_error(
                        "Directory is not empty",
                        &message,
                        suggestions
                    ).map_err(|e| MoonflareError::file_system_error("UI render", target_dir.clone(), std::io::Error::other(e.to_string()))).into_diagnostic()?;
                    
                    // Return early with exit code 1 - the UI has already shown the nice error
                    // Use a minimal error since our UI already displayed the helpful message
                    std::process::exit(1);
                } else if !entries.is_empty() && force {
                    self.ui.render_section_start(&format!("Warning: Directory '{}' contains {} files. Proceeding with --force.", target_dir.display(), entries.len()))
                        .map_err(|e| MoonflareError::file_system_error("UI render", target_dir.clone(), std::io::Error::other(e.to_string()))).into_diagnostic()?;
                }
            } else {
                // Path exists but is not a directory
                return Err(MoonflareError::workspace_directory_exists(target_dir)).into_diagnostic();
            }
        }

        // Create directory with better error handling
        create_directory_if_not_exists(&target_dir)
            .map_err(|e| {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            MoonflareError::permission_denied(target_dir.clone(), std::io::Error::new(io_err.kind(), format!("{}", io_err)))
                        }
                        _ => MoonflareError::file_system_error("create directory", target_dir.clone(), std::io::Error::new(io_err.kind(), format!("{}", io_err)))
                    }
                } else {
                    MoonflareError::file_system_error("create directory", target_dir.clone(), 
                        std::io::Error::other(e.to_string()))
                }
            })
            .into_diagnostic()?;
        
        // Check if Moon is installed
        check_moon_installation()
            .map_err(|e| MoonflareError::moon_not_found(Some(e.to_string())))
            .into_diagnostic()?;

        // Prepare template context
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::String(workspace_name.clone()));

        // Generate workspace files
        if let Some(template) = embedded::get_template("workspace") {
            self.template_engine.process_template_files(
                template,
                &target_dir,
                &context
            ).map_err(|e| MoonflareError::template_error("workspace", Box::new(std::io::Error::other(e.to_string()))))
            .into_diagnostic()?;
        }

        // Create directory structure
        let dirs = ["apps", "sites", "workers", "crates"];
        for dir in dirs {
            create_directory_if_not_exists(&target_dir.join(dir))
                .map_err(|e| {
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        MoonflareError::file_system_error(
                            &format!("create {} directory", dir), 
                            target_dir.join(dir), 
                            std::io::Error::new(io_err.kind(), format!("{}", io_err))
                        )
                    } else {
                        MoonflareError::file_system_error(
                            &format!("create {} directory", dir), 
                            target_dir.join(dir), 
                            std::io::Error::other(e.to_string())
                        )
                    }
                })
                .into_diagnostic()?;
        }

        // Run moon setup in the new workspace
        self.ui.render_section_start("Initializing Moon workspace")
            .map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;
        let current_dir = std::env::current_dir()
            .map_err(|e| MoonflareError::file_system_error("get current directory", std::env::current_dir().unwrap_or_default(), e))
            .into_diagnostic()?;
        
        std::env::set_current_dir(&target_dir)
            .map_err(|e| MoonflareError::file_system_error("change directory", target_dir.clone(), e))
            .into_diagnostic()?;
        
        match moon_setup().await {
            Ok(_) => {
                self.ui.render_success("Moon workspace initialized")
                    .map_err(|e| MoonflareError::file_system_error("UI render", target_dir.clone(), std::io::Error::other(e.to_string()))).into_diagnostic()?;
            }
            Err(e) => {
                // Restore directory before potentially returning error
                let _ = std::env::set_current_dir(&current_dir);
                
                // Use proper Moon setup error for better diagnostics
                return Err(MoonflareError::moon_setup_failed(
                    target_dir.clone(), 
                    Box::new(std::io::Error::other(e.to_string())), 
                    None
                )).into_diagnostic();
            }
        }
        
        // Restore original directory
        std::env::set_current_dir(&current_dir)
            .map_err(|e| MoonflareError::file_system_error("restore directory", current_dir.clone(), e))
            .into_diagnostic()?;

        self.ui.render_success(&format!("Successfully created {} monorepo!", workspace_name))
            .map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;

        self.ui.render_workspace_structure()
            .map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;

        let mut steps = vec![];
        if name != "." {
            steps.push(format!("cd {}", workspace_name));
        }
        steps.push("moonflare add <type> <name>  # Add a new project".to_string());
        
        let step_refs: Vec<&str> = steps.iter().map(|s| s.as_str()).collect();
        self.ui.render_next_steps(step_refs)
            .map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;

        self.ui.render_project_types()
            .map_err(|e| MoonflareError::file_system_error("UI render", std::env::current_dir().unwrap_or_default(), std::io::Error::other(e.to_string()))).into_diagnostic()?;

        Ok(())
    }
}