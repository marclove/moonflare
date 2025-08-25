use super::{log, MoonflareTestWorkspace, ProjectType};

// Verification functions
impl MoonflareTestWorkspace {
    pub fn verify_typescript_project_has_wasm_deps(
        &self,
        workspace_name: &str,
        project_type: &ProjectType,
        project_name: &str,
        should_have_deps: bool,
    ) -> anyhow::Result<()> {
        log(&format!(
            "Verifying WASM dependencies for {} project (expected: {})",
            project_type.as_str(),
            should_have_deps
        ));

        let moon_yml_path = self
            .path()
            .join(workspace_name)
            .join(project_type.directory())
            .join(project_name)
            .join("moon.yml");

        // Fast-fail if file doesn't exist
        if !moon_yml_path.exists() {
            anyhow::bail!("moon.yml not found at {:?}", moon_yml_path);
        }

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
            log("Project correctly has WASM dependencies");
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
            log("Project correctly has no WASM dependencies");
        }

        Ok(())
    }

    pub fn verify_shared_wasm_has_crate_deps(
        &self,
        workspace_name: &str,
        crate_names: &[String],
    ) -> anyhow::Result<()> {
        log(&format!(
            "Verifying shared-wasm dependencies: {:?}",
            crate_names
        ));

        let shared_wasm_moon_yml = self
            .path()
            .join(workspace_name)
            .join("shared-wasm")
            .join("moon.yml");

        // Fast-fail if file doesn't exist
        if !shared_wasm_moon_yml.exists() {
            anyhow::bail!(
                "shared-wasm/moon.yml not found at {:?}",
                shared_wasm_moon_yml
            );
        }

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

        log(&format!(
            "Verified {} shared-wasm dependencies",
            crate_names.len()
        ));
        Ok(())
    }

    pub fn verify_wasm_files_exist(
        &self,
        workspace_name: &str,
        crate_names: &[String],
    ) -> anyhow::Result<()> {
        log(&format!(
            "Verifying WASM files in shared-wasm: {:?}",
            crate_names
        ));
        let shared_wasm_dir = self.path().join(workspace_name).join("shared-wasm");

        // Fast-fail if directory doesn't exist
        if !shared_wasm_dir.exists() {
            anyhow::bail!("shared-wasm directory not found at {:?}", shared_wasm_dir);
        }

        for crate_name in crate_names {
            // Rust converts dashes to underscores in WASM filenames
            let wasm_filename = crate_name.replace('-', "_");
            let wasm_file = shared_wasm_dir.join(format!("{}.wasm", wasm_filename));
            if !wasm_file.exists() {
                log(&format!("Missing WASM file: {:?}", wasm_file));
                anyhow::bail!(
                    "WASM file for crate {} should exist at {:?} but doesn't",
                    crate_name,
                    wasm_file
                );
            }
            log(&format!("Found WASM file: {}.wasm", wasm_filename));
        }

        Ok(())
    }

    pub fn verify_wasm_files_in_typescript_dist(
        &self,
        workspace_name: &str,
        project_type: &ProjectType,
        project_name: &str,
        crate_names: &[String],
    ) -> anyhow::Result<()> {
        log(&format!(
            "Verifying WASM files in {} project dist: {:?}",
            project_type.as_str(),
            crate_names
        ));

        let dist_dir = self
            .path()
            .join(workspace_name)
            .join(project_type.directory())
            .join(project_name)
            .join("dist");

        // Fast-fail if dist directory doesn't exist
        if !dist_dir.exists() {
            log(&format!("Missing dist directory: {:?}", dist_dir));
            anyhow::bail!(
                "Dist directory for {} project {} should exist at {:?} but doesn't",
                project_type.as_str(),
                project_name,
                dist_dir
            );
        }

        for crate_name in crate_names {
            // Rust converts dashes to underscores in WASM filenames
            let wasm_filename = crate_name.replace('-', "_");
            let wasm_file = dist_dir.join(format!("{}.wasm", wasm_filename));
            if !wasm_file.exists() {
                log(&format!("Missing WASM file in dist: {:?}", wasm_file));
                anyhow::bail!(
                    "WASM file for crate {} should exist in {} project {} dist at {:?} but doesn't",
                    crate_name,
                    project_type.as_str(),
                    project_name,
                    wasm_file
                );
            }
            log(&format!(
                "Found WASM file in {} dist: {}.wasm",
                project_type.as_str(),
                wasm_filename
            ));
        }

        Ok(())
    }
}