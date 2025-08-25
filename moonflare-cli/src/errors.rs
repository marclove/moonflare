use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum MoonflareError {
    #[error("Invalid workspace name")]
    #[diagnostic(
        code(moonflare::init::invalid_name),
        help("Workspace names should use lowercase letters, numbers, and hyphens only. Examples: 'my-app', 'website', 'api-server'"),
        url("https://moonflare.dev/docs/workspaces#naming")
    )]
    InvalidWorkspaceName {
        #[source_code]
        name: NamedSource<String>,
        #[label("This name contains invalid characters")]
        invalid_span: SourceSpan,
        suggestions: Vec<String>,
    },

    #[error("Workspace directory already exists")]
    #[diagnostic(
        code(moonflare::init::directory_exists),
        help("Choose a different name or remove the existing directory first")
    )]
    WorkspaceDirectoryExists {
        path: String,
        suggestion: String,
    },

    #[error("Permission denied")]
    #[diagnostic(
        code(moonflare::init::permission_denied),
        help("Make sure you have write permissions to the parent directory")
    )]
    PermissionDenied {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Moon CLI not found")]
    #[diagnostic(
        code(moonflare::init::moon_not_found),
        help("Install Moon CLI manually with: curl -fsSL https://moonrepo.dev/install/moon.sh | bash"),
        url("https://moonrepo.dev/docs/install")
    )]
    MoonNotFound {
        auto_install_failed: Option<String>,
    },

    #[error("Template processing failed")]
    #[diagnostic(
        code(moonflare::init::template_error),
        help("This is likely a bug in Moonflare. Please report it at https://github.com/moonflare-dev/moonflare/issues")
    )]
    TemplateError {
        template_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Moon workspace setup failed")]
    #[diagnostic(
        code(moonflare::init::moon_setup_failed),
        help("You can run 'moon setup' manually in the workspace directory")
    )]
    MoonSetupFailed {
        workspace_path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        moon_output: Option<String>,
    },

    #[error("Moon command failed")]
    #[diagnostic(
        code(moonflare::moon::command_failed),
        help("Make sure Moon CLI is properly installed and the workspace is valid")
    )]
    MoonCommandFailed {
        command: String,
        #[source_code]
        stderr_output: NamedSource<String>,
        #[label("Error occurred here")]
        error_span: Option<SourceSpan>,
        exit_code: Option<i32>,
    },

    #[error("Not in a Moonflare workspace")]
    #[diagnostic(
        code(moonflare::build::not_in_workspace),
        help("Navigate to a Moonflare workspace directory or create one with 'moonflare init <name>'")
    )]
    NotInWorkspace {
        current_dir: String,
        searched_paths: Vec<String>,
    },

    #[error("Project not found")]
    #[diagnostic(
        code(moonflare::build::project_not_found),
        help("List available projects with 'moon query projects'")
    )]
    ProjectNotFound {
        project_name: String,
        workspace_path: String,
        available_projects: Option<String>,
    },



    #[error("File system error")]
    #[diagnostic(
        code(moonflare::fs::operation_failed),
        help("Check file permissions and available disk space")
    )]
    FileSystemError {
        operation: String,
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl MoonflareError {
    pub fn invalid_workspace_name(name: &str, suggestions: Vec<String>) -> Self {
        let name_source = NamedSource::new("workspace_name", name.to_string());
        let invalid_span = SourceSpan::new(0.into(), name.len());
        
        Self::InvalidWorkspaceName {
            name: name_source,
            invalid_span,
            suggestions,
        }
    }

    pub fn workspace_directory_exists(path: PathBuf) -> Self {
        let path_str = path.display().to_string();
        let suggestion = if path.exists() && path.is_dir() {
            format!("Remove existing directory: rm -rf {}", path.display())
        } else {
            "Choose a different workspace name".to_string()
        };

        Self::WorkspaceDirectoryExists { path: path_str, suggestion }
    }

    pub fn permission_denied(path: PathBuf, source: std::io::Error) -> Self {
        Self::PermissionDenied { path: path.display().to_string(), source }
    }

    pub fn moon_not_found(auto_install_error: Option<String>) -> Self {
        Self::MoonNotFound {
            auto_install_failed: auto_install_error,
        }
    }

    pub fn template_error(template_name: &str, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::TemplateError {
            template_name: template_name.to_string(),
            source,
        }
    }


    pub fn not_in_workspace(current_dir: PathBuf, searched_paths: Vec<PathBuf>) -> Self {
        Self::NotInWorkspace {
            current_dir: current_dir.display().to_string(),
            searched_paths: searched_paths.iter().map(|p| p.display().to_string()).collect(),
        }
    }

    pub fn project_not_found(project_name: &str, workspace_path: PathBuf, available_projects: Option<String>) -> Self {
        Self::ProjectNotFound {
            project_name: project_name.to_string(),
            workspace_path: workspace_path.display().to_string(),
            available_projects,
        }
    }


    pub fn moon_setup_failed(workspace_path: PathBuf, source: Box<dyn std::error::Error + Send + Sync>, moon_output: Option<String>) -> Self {
        Self::MoonSetupFailed {
            workspace_path: workspace_path.display().to_string(),
            source,
            moon_output,
        }
    }

    pub fn moon_command_failed(command: &str, stderr_output: &str, exit_code: Option<i32>) -> Self {
        let stderr_source = NamedSource::new("moon_stderr", stderr_output.to_string());
        let error_span = find_error_span(stderr_output);
        
        Self::MoonCommandFailed {
            command: command.to_string(),
            stderr_output: stderr_source,
            error_span,
            exit_code,
        }
    }

    pub fn file_system_error(operation: &str, path: PathBuf, source: std::io::Error) -> Self {
        Self::FileSystemError {
            operation: operation.to_string(),
            path: path.display().to_string(),
            source,
        }
    }
}

fn find_error_span(output: &str) -> Option<SourceSpan> {
    // Look for common error patterns and return their spans
    let error_patterns = [
        "error:",
        "Error:",
        "ERROR:",
        "failed:",
        "Failed:",
        "FAILED:",
        "panic:",
        "Panic:",
        "PANIC:",
    ];
    
    for pattern in &error_patterns {
        if let Some(pos) = output.find(pattern) {
            // Find the end of the error line
            let end_pos = output[pos..]
                .find('\n')
                .map(|n| pos + n)
                .unwrap_or(output.len());
            
            return Some(SourceSpan::new(pos.into(), end_pos - pos));
        }
    }
    
    None
}

pub fn validate_workspace_name(name: &str) -> Result<(), MoonflareError> {
    let mut suggestions = Vec::new();
    let mut has_issues = false;

    if name.is_empty() {
        return Err(MoonflareError::invalid_workspace_name(name, vec!["my-app".to_string()]));
    }

    // Check for invalid characters
    let valid_chars = name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid_chars {
        has_issues = true;
        let suggestion = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        if !suggestion.is_empty() && suggestion != name {
            suggestions.push(suggestion);
        }
    }

    // Check for leading/trailing hyphens
    if name.starts_with('-') || name.ends_with('-') {
        has_issues = true;
        suggestions.push(name.trim_matches('-').to_string());
    }

    // Check for double hyphens
    if name.contains("--") {
        has_issues = true;
        suggestions.push(name.replace("--", "-"));
    }

    // Check length
    if name.len() > 100 {
        has_issues = true;
        suggestions.push(name.chars().take(50).collect());
    }

    if has_issues {
        // Remove duplicates and empty suggestions
        suggestions.sort();
        suggestions.dedup();
        suggestions.retain(|s| !s.is_empty() && s != name);
        
        if suggestions.is_empty() {
            suggestions.push("my-project".to_string());
        }
        
        return Err(MoonflareError::invalid_workspace_name(name, suggestions));
    }

    Ok(())
}