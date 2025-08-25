use proptest::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// Project types that can be added
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum ProjectType {
    Astro,
    React,
    DurableObject,
    Crate,
}

impl ProjectType {
    fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Astro => "astro",
            ProjectType::React => "react",
            ProjectType::DurableObject => "durable-object",
            ProjectType::Crate => "crate",
        }
    }

    fn is_typescript(&self) -> bool {
        matches!(
            self,
            ProjectType::Astro | ProjectType::React | ProjectType::DurableObject
        )
    }

    fn is_crate(&self) -> bool {
        matches!(self, ProjectType::Crate)
    }

    fn directory(&self) -> &'static str {
        match self {
            ProjectType::Astro => "sites",
            ProjectType::React => "apps",
            ProjectType::DurableObject => "workers",
            ProjectType::Crate => "crates",
        }
    }
}

// A project addition operation
#[derive(Debug, Clone)]
struct ProjectAdd {
    project_type: ProjectType,
    name: String,
}

// Test fixture that manages a temporary moonflare workspace
struct MoonflareTestWorkspace {
    temp_dir: TempDir,
    moonflare_binary: PathBuf,
}

impl MoonflareTestWorkspace {
    fn new() -> anyhow::Result<Self> {
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

    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn init(&self, name: &str) -> anyhow::Result<()> {
        let output = Command::new(&self.moonflare_binary)
            .arg("init")
            .arg(name)
            .current_dir(self.temp_dir.path())
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to init moonflare workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn add_project(
        &self,
        workspace_name: &str,
        project_type: &ProjectType,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let output = Command::new(&self.moonflare_binary)
            .arg("add")
            .arg(project_type.as_str())
            .arg(project_name)
            .current_dir(self.temp_dir.path().join(workspace_name))
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to add {} project '{}': {}",
                project_type.as_str(),
                project_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn build(&self, workspace_name: &str) -> anyhow::Result<()> {
        let output = Command::new(&self.moonflare_binary)
            .arg("build")
            .current_dir(self.temp_dir.path().join(workspace_name))
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to build workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}

// Verification functions
impl MoonflareTestWorkspace {
    fn verify_typescript_project_has_wasm_deps(
        &self,
        workspace_name: &str,
        project_type: &ProjectType,
        project_name: &str,
        should_have_deps: bool,
    ) -> anyhow::Result<()> {
        let moon_yml_path = self
            .path()
            .join(workspace_name)
            .join(project_type.directory())
            .join(project_name)
            .join("moon.yml");

        let content = std::fs::read_to_string(&moon_yml_path)?;
        let config: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let has_wasm_deps = config
            .get("tasks")
            .and_then(|tasks| tasks.get("build"))
            .and_then(|build| build.get("deps"))
            .and_then(|deps| deps.as_sequence())
            .map(|deps| {
                deps.iter()
                    .any(|dep| dep.as_str() == Some("shared-wasm:gather"))
            })
            .unwrap_or(false);

        let has_wasm_inputs = config
            .get("tasks")
            .and_then(|tasks| tasks.get("build"))
            .and_then(|build| build.get("inputs"))
            .and_then(|inputs| inputs.as_sequence())
            .map(|inputs| {
                inputs.iter().any(|input| {
                    input
                        .as_str()
                        .map(|s| s.contains("shared-wasm"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if should_have_deps {
            if !has_wasm_deps {
                anyhow::bail!(
                    "TypeScript project {} should have shared-wasm:gather dependency but doesn't",
                    project_name
                );
            }
            if !has_wasm_inputs {
                anyhow::bail!(
                    "TypeScript project {} should have shared-wasm inputs but doesn't",
                    project_name
                );
            }
        } else {
            if has_wasm_deps {
                anyhow::bail!(
                    "TypeScript project {} should NOT have shared-wasm:gather dependency but does",
                    project_name
                );
            }
            if has_wasm_inputs {
                anyhow::bail!(
                    "TypeScript project {} should NOT have shared-wasm inputs but does",
                    project_name
                );
            }
        }

        Ok(())
    }

    fn verify_shared_wasm_has_crate_deps(
        &self,
        workspace_name: &str,
        crate_names: &[String],
    ) -> anyhow::Result<()> {
        let shared_wasm_moon_yml = self
            .path()
            .join(workspace_name)
            .join("shared-wasm")
            .join("moon.yml");

        let content = std::fs::read_to_string(&shared_wasm_moon_yml)?;
        let config: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let empty_deps = vec![];
        let deps = config
            .get("tasks")
            .and_then(|tasks| tasks.get("gather"))
            .and_then(|gather| gather.get("deps"))
            .and_then(|deps| deps.as_sequence())
            .unwrap_or(&empty_deps);

        for crate_name in crate_names {
            let expected_dep = format!("{}:build", crate_name);
            let has_dep = deps.iter().any(|dep| dep.as_str() == Some(&expected_dep));
            if !has_dep {
                anyhow::bail!(
                    "shared-wasm:gather should depend on {} but doesn't",
                    expected_dep
                );
            }
        }

        Ok(())
    }

    fn verify_wasm_files_exist(
        &self,
        workspace_name: &str,
        crate_names: &[String],
    ) -> anyhow::Result<()> {
        let shared_wasm_dir = self.path().join(workspace_name).join("shared-wasm");

        for crate_name in crate_names {
            // Rust converts dashes to underscores in WASM filenames
            let wasm_filename = crate_name.replace('-', "_");
            let wasm_file = shared_wasm_dir.join(format!("{}.wasm", wasm_filename));
            if !wasm_file.exists() {
                anyhow::bail!(
                    "WASM file for crate {} should exist at {:?} but doesn't",
                    crate_name,
                    wasm_file
                );
            }
        }

        Ok(())
    }
}

// Property test generators
prop_compose! {
    fn arb_project_type()(project_type in prop_oneof![
        Just(ProjectType::Astro),
        Just(ProjectType::React),
        Just(ProjectType::DurableObject),
        Just(ProjectType::Crate),
    ]) -> ProjectType {
        project_type
    }
}

prop_compose! {
    fn arb_project_name()(name in "[a-z][a-z0-9-]{4,12}") -> String {
        name
    }
}

prop_compose! {
    fn arb_project_add()(
        project_type in arb_project_type(),
        name in arb_project_name()
    ) -> ProjectAdd {
        ProjectAdd { project_type, name }
    }
}

prop_compose! {
    fn arb_project_sequence()(
        projects in prop::collection::vec(arb_project_add(), 1..5)
    ) -> Vec<ProjectAdd> {
        // Ensure unique names
        let mut seen_names = std::collections::HashSet::new();
        let mut unique_projects = Vec::new();

        for mut project in projects {
            let mut counter = 0;
            let base_name = project.name.clone();

            while seen_names.contains(&project.name) {
                project.name = format!("{}{}", base_name, counter);
                counter += 1;
            }

            seen_names.insert(project.name.clone());
            unique_projects.push(project);
        }

        unique_projects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_workspace_creation() -> anyhow::Result<()> {
        let workspace = MoonflareTestWorkspace::new()?;
        workspace.init("test-workspace")?;

        // Verify workspace structure exists
        let workspace_path = workspace.path().join("test-workspace");
        assert!(workspace_path.join(".moon").exists());
        assert!(workspace_path.join("package.json").exists());
        assert!(workspace_path.join("shared-wasm").exists());

        Ok(())
    }

    #[test]
    fn test_typescript_project_without_crates() -> anyhow::Result<()> {
        let workspace = MoonflareTestWorkspace::new()?;
        workspace.init("test-workspace")?;

        workspace.add_project("test-workspace", &ProjectType::Astro, "marketing")?;

        // Verify TypeScript project has NO WASM dependencies
        workspace.verify_typescript_project_has_wasm_deps(
            "test-workspace",
            &ProjectType::Astro,
            "marketing",
            false,
        )?;

        // Should build successfully
        workspace.build("test-workspace")?;

        Ok(())
    }

    #[test]
    fn test_crate_then_typescript() -> anyhow::Result<()> {
        let workspace = MoonflareTestWorkspace::new()?;
        workspace.init("test-workspace")?;

        // Add TypeScript project first
        workspace.add_project("test-workspace", &ProjectType::Astro, "dashboard")?;
        workspace.verify_typescript_project_has_wasm_deps(
            "test-workspace",
            &ProjectType::Astro,
            "dashboard",
            false,
        )?;

        // Add crate - should update existing TypeScript project
        workspace.add_project("test-workspace", &ProjectType::Crate, "math")?;

        // Verify TypeScript project now has WASM dependencies
        workspace.verify_typescript_project_has_wasm_deps(
            "test-workspace",
            &ProjectType::Astro,
            "dashboard",
            true,
        )?;

        // Verify shared-wasm depends on crate
        workspace.verify_shared_wasm_has_crate_deps("test-workspace", &["math".to_string()])?;

        // Build and verify WASM files
        workspace.build("test-workspace")?;
        workspace.verify_wasm_files_exist("test-workspace", &["math".to_string()])?;

        Ok(())
    }

    #[test]
    fn test_typescript_after_crates_exist() -> anyhow::Result<()> {
        let workspace = MoonflareTestWorkspace::new()?;
        workspace.init("test-workspace")?;

        // Add crate first
        workspace.add_project("test-workspace", &ProjectType::Crate, "utils")?;

        // Add TypeScript project - should automatically get WASM dependencies
        workspace.add_project("test-workspace", &ProjectType::DurableObject, "api")?;

        workspace.verify_typescript_project_has_wasm_deps(
            "test-workspace",
            &ProjectType::DurableObject,
            "api",
            true,
        )?;

        workspace.verify_shared_wasm_has_crate_deps("test-workspace", &["utils".to_string()])?;

        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]
        #[test]
        fn test_property_based_project_sequences(
            projects in arb_project_sequence()
        ) {
            let workspace = MoonflareTestWorkspace::new().unwrap();
            workspace.init("test-workspace").unwrap();

            let mut crates_added = Vec::new();
            let mut typescript_projects = Vec::new();

            // Add all projects in sequence
            for project in &projects {
                workspace.add_project("test-workspace", &project.project_type, &project.name).unwrap();

                if project.project_type.is_crate() {
                    crates_added.push(project.name.clone());
                } else if project.project_type.is_typescript() {
                    typescript_projects.push(project.clone());
                }
            }

            // Verify final state
            let should_have_wasm_deps = !crates_added.is_empty();

            for ts_project in &typescript_projects {
                workspace.verify_typescript_project_has_wasm_deps(
                    "test-workspace",
                    &ts_project.project_type,
                    &ts_project.name,
                    should_have_wasm_deps
                ).unwrap();
            }

            if !crates_added.is_empty() {
                workspace.verify_shared_wasm_has_crate_deps("test-workspace", &crates_added).unwrap();
            }

            // Build should always succeed
            workspace.build("test-workspace").unwrap();

            // If we have crates AND TypeScript projects, verify WASM files exist
            if !crates_added.is_empty() && !typescript_projects.is_empty() {
                workspace.verify_wasm_files_exist("test-workspace", &crates_added).unwrap();
            }
        }
    }
}
