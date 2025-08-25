use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

use super::{ProjectType, log, run_command_with_timeout};

// Test fixture that manages a temporary moonflare workspace
pub struct MoonflareTestWorkspace {
    temp_dir: TempDir,
    moonflare_binary: PathBuf,
}

impl MoonflareTestWorkspace {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;

        // Get the path to the moonflare binary
        let moonflare_binary = std::env::current_dir()?
            .join("target")
            .join("release")
            .join("moonflare");

        // Ensure the binary exists
        if !moonflare_binary.exists() {
            anyhow::bail!(
                "Moonflare binary not found at {:?}. Run 'cargo build --release' first.",
                moonflare_binary
            );
        }

        Ok(Self {
            temp_dir,
            moonflare_binary,
        })
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn moonflare_binary(&self) -> &PathBuf {
        &self.moonflare_binary
    }

    pub fn init(&self, name: &str) -> anyhow::Result<()> {
        let start = Instant::now();
        log(&format!("Initializing workspace: {}", name));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("init").arg(name).current_dir(self.temp_dir.path());

        let output = run_command_with_timeout(cmd, 5)?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to init moonflare workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        log(&format!("Workspace initialized in {:?}", start.elapsed()));
        Ok(())
    }

    pub fn init_with_force(&self, name: &str) -> anyhow::Result<()> {
        let start = Instant::now();
        log(&format!("Initializing workspace with force: {}", name));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("init")
            .arg(name)
            .arg("--force")
            .current_dir(self.temp_dir.path());

        let output = run_command_with_timeout(cmd, 5)?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to init moonflare workspace with force: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        log(&format!(
            "Workspace initialized with force in {:?}",
            start.elapsed()
        ));
        Ok(())
    }

    pub fn init_should_fail(&self, name: &str) -> anyhow::Result<String> {
        let start = Instant::now();
        log(&format!("Expecting init to fail for: {}", name));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("init").arg(name).current_dir(self.temp_dir.path());

        let output = run_command_with_timeout(cmd, 5)?;

        if output.status.success() {
            anyhow::bail!("Expected init to fail, but it succeeded");
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        log(&format!(
            "Init failed as expected in {:?}: {}",
            start.elapsed(),
            stderr.lines().next().unwrap_or("Unknown error")
        ));
        Ok(stderr)
    }

    pub fn create_subdirectory(&self, name: &str) -> anyhow::Result<PathBuf> {
        let subdir_path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&subdir_path)?;
        Ok(subdir_path)
    }

    pub fn create_file_in_directory(
        &self,
        dir: &Path,
        filename: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        let file_path = dir.join(filename);
        std::fs::write(file_path, content)?;
        Ok(())
    }

    pub fn add_project(
        &self,
        workspace_name: &str,
        project_type: &ProjectType,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let start = Instant::now();
        log(&format!(
            "Adding {} project: {}",
            project_type.as_str(),
            project_name
        ));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("add")
            .arg(project_type.as_str())
            .arg(project_name)
            .current_dir(self.temp_dir.path().join(workspace_name));

        let output = run_command_with_timeout(cmd, 5)?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to add {} project '{}': {}",
                project_type.as_str(),
                project_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        log(&format!(
            "Added {} project in {:?}",
            project_type.as_str(),
            start.elapsed()
        ));
        Ok(())
    }

    pub fn build(&self, workspace_name: &str) -> anyhow::Result<()> {
        let start = Instant::now();
        log(&format!("Building workspace '{}'", workspace_name));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("build")
            .current_dir(self.temp_dir.path().join(workspace_name));

        let output = run_command_with_timeout(cmd, 45)?;

        if !output.status.success() {
            log(&format!("Build failed after {:?}", start.elapsed()));
            log(&format!(
                "STDOUT: {}",
                String::from_utf8_lossy(&output.stdout)
            ));
            log(&format!(
                "STDERR: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
            anyhow::bail!(
                "Failed to build workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        log(&format!("Build completed in {:?}", start.elapsed()));
        Ok(())
    }

    pub fn deploy(&self, workspace_name: &str) -> anyhow::Result<()> {
        let start = Instant::now();
        log(&format!("Deploying workspace '{}'", workspace_name));

        let mut cmd = Command::new(&self.moonflare_binary);
        cmd.arg("deploy")
            .current_dir(self.temp_dir.path().join(workspace_name));

        let output = run_command_with_timeout(cmd, 300)?; // 5 minutes timeout

        if !output.status.success() {
            log(&format!("Deploy failed after {:?}", start.elapsed()));
            log(&format!(
                "STDOUT: {}",
                String::from_utf8_lossy(&output.stdout)
            ));
            log(&format!(
                "STDERR: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
            anyhow::bail!(
                "Failed to deploy workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        log(&format!("Deploy completed in {:?}", start.elapsed()));
        Ok(())
    }
}
