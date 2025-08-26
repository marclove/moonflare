use common::*;
use std::fs;

mod common;

#[test]
fn test_rename_react_project_updates_directory_and_wrangler_jsonc() -> anyhow::Result<()> {
    log("→ Rename React Project Updates Directory and Wrangler JSONC");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::React, "frontend")?;

    let project_path = workspace.path().join("test-project");
    let old_path = project_path.join("apps").join("frontend");
    let new_path = project_path.join("apps").join("web-app");

    // Verify original project exists
    assert!(old_path.is_dir(), "Original frontend directory should exist");
    let wrangler_config = old_path.join("wrangler.jsonc");
    assert!(wrangler_config.is_file(), "wrangler.jsonc should exist");

    // Check original config content
    let original_content = fs::read_to_string(&wrangler_config)?;
    assert!(original_content.contains(r#""name": "frontend""#), "Original config should have frontend name");

    // Rename the project
    workspace.rename_project("test-project", "frontend", "web-app")?;

    // Verify directory was renamed
    assert!(!old_path.exists(), "Old frontend directory should not exist");
    assert!(new_path.is_dir(), "New web-app directory should exist");

    // Verify wrangler config was updated
    let wrangler_config_new = new_path.join("wrangler.jsonc");
    assert!(wrangler_config_new.is_file(), "wrangler.jsonc should exist in new location");

    let updated_content = fs::read_to_string(&wrangler_config_new)?;
    assert!(updated_content.contains(r#""name": "web-app""#), "Updated config should have web-app name");
    assert!(!updated_content.contains(r#""name": "frontend""#), "Updated config should not have old name");

    // Verify other files were preserved
    assert!(new_path.join("package.json").is_file(), "package.json should be preserved");
    assert!(new_path.join("src").is_dir(), "src directory should be preserved");

    Ok(())
}

#[test]
fn test_rename_durable_object_updates_directory_and_wrangler_toml() -> anyhow::Result<()> {
    log("→ Rename Durable Object Updates Directory and Wrangler TOML");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::DurableObject, "api")?;

    let project_path = workspace.path().join("test-project");
    let old_path = project_path.join("workers").join("api");
    let new_path = project_path.join("workers").join("backend");

    // Verify original project exists
    assert!(old_path.is_dir(), "Original api directory should exist");
    let wrangler_config = old_path.join("wrangler.toml");
    assert!(wrangler_config.is_file(), "wrangler.toml should exist");

    // Check original config content
    let original_content = fs::read_to_string(&wrangler_config)?;
    assert!(original_content.contains(r#"name = "api""#), "Original config should have api name");

    // Rename the project
    workspace.rename_project("test-project", "api", "backend")?;

    // Verify directory was renamed
    assert!(!old_path.exists(), "Old api directory should not exist");
    assert!(new_path.is_dir(), "New backend directory should exist");

    // Verify wrangler config was updated
    let wrangler_config_new = new_path.join("wrangler.toml");
    assert!(wrangler_config_new.is_file(), "wrangler.toml should exist in new location");

    let updated_content = fs::read_to_string(&wrangler_config_new)?;
    assert!(updated_content.contains(r#"name = "backend""#), "Updated config should have backend name");
    assert!(!updated_content.contains(r#"name = "api""#), "Updated config should not have old name");

    // Verify other content was preserved
    assert!(updated_content.contains("[[durable_objects.bindings]]"), "Durable object bindings should be preserved");

    Ok(())
}

#[test]
fn test_rename_astro_project_updates_directory_and_wrangler_jsonc() -> anyhow::Result<()> {
    log("→ Rename Astro Project Updates Directory and Wrangler JSONC");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::Astro, "site")?;

    let project_path = workspace.path().join("test-project");
    let old_path = project_path.join("sites").join("site");
    let new_path = project_path.join("sites").join("marketing");

    // Verify original project exists
    assert!(old_path.is_dir(), "Original site directory should exist");
    let wrangler_config = old_path.join("wrangler.jsonc");
    assert!(wrangler_config.is_file(), "wrangler.jsonc should exist");

    // Rename the project
    workspace.rename_project("test-project", "site", "marketing")?;

    // Verify directory was renamed
    assert!(!old_path.exists(), "Old site directory should not exist");
    assert!(new_path.is_dir(), "New marketing directory should exist");

    // Verify wrangler config was updated
    let wrangler_config_new = new_path.join("wrangler.jsonc");
    assert!(wrangler_config_new.is_file(), "wrangler.jsonc should exist in new location");

    let updated_content = fs::read_to_string(&wrangler_config_new)?;
    assert!(updated_content.contains(r#""name": "marketing""#), "Updated config should have marketing name");
    assert!(!updated_content.contains(r#""name": "site""#), "Updated config should not have old name");

    Ok(())
}

#[test]
fn test_rename_crate_project_updates_directory_only() -> anyhow::Result<()> {
    log("→ Rename Crate Project Updates Directory Only");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::Crate, "utils")?;

    let project_path = workspace.path().join("test-project");
    let old_path = project_path.join("crates").join("utils");
    let new_path = project_path.join("crates").join("core");

    // Verify original project exists
    assert!(old_path.is_dir(), "Original utils directory should exist");
    assert!(old_path.join("Cargo.toml").is_file(), "Cargo.toml should exist");

    // Rename the project
    workspace.rename_project("test-project", "utils", "core")?;

    // Verify directory was renamed
    assert!(!old_path.exists(), "Old utils directory should not exist");
    assert!(new_path.is_dir(), "New core directory should exist");

    // Verify Rust files were preserved
    assert!(new_path.join("Cargo.toml").is_file(), "Cargo.toml should be preserved");
    assert!(new_path.join("src").is_dir(), "src directory should be preserved");
    assert!(new_path.join("src").join("lib.rs").is_file(), "lib.rs should be preserved");

    // Verify no wrangler files (crates don't have them)
    assert!(!new_path.join("wrangler.toml").exists(), "Crates should not have wrangler.toml");
    assert!(!new_path.join("wrangler.json").exists(), "Crates should not have wrangler.json");
    assert!(!new_path.join("wrangler.jsonc").exists(), "Crates should not have wrangler.jsonc");

    Ok(())
}

#[test]
fn test_rename_project_fails_when_project_not_found() -> anyhow::Result<()> {
    log("→ Rename Project Fails When Project Not Found");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;

    // Try to rename a non-existent project
    let stderr = workspace.rename_project_should_fail("test-project", "nonexistent", "newname")?;

    // Check error message
    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("project 'nonexistent' not found"),
        "Error should mention project not found, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_rename_project_fails_when_target_name_exists_in_same_directory() -> anyhow::Result<()> {
    log("→ Rename Project Fails When Target Name Exists In Same Directory");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::React, "frontend")?;
    workspace.add_project("test-project", &ProjectType::React, "backend")?;

    // Try to rename frontend to backend (both React projects)
    let stderr = workspace.rename_project_should_fail("test-project", "frontend", "backend")?;

    // Check error message
    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("already exists") && stderr_lower.contains("backend"),
        "Error should mention project already exists, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_rename_allows_same_name_across_different_project_types() -> anyhow::Result<()> {
    log("→ Rename Allows Same Name Across Different Project Types");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::React, "core")?;
    workspace.add_project("test-project", &ProjectType::Crate, "utils")?;

    let project_path = workspace.path().join("test-project");

    // Rename utils crate to core (different directory from React core)
    workspace.rename_project("test-project", "utils", "core")?;

    // Verify both core projects exist in different directories
    assert!(project_path.join("apps").join("core").is_dir(), "React core should exist");
    assert!(project_path.join("crates").join("core").is_dir(), "Crate core should exist");
    
    Ok(())
}

#[test]
fn test_rename_preserves_wrangler_json_format() -> anyhow::Result<()> {
    log("→ Rename Preserves Wrangler JSON Format");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    workspace.add_project("test-project", &ProjectType::React, "frontend")?;

    let project_path = workspace.path().join("test-project");
    let frontend_path = project_path.join("apps").join("frontend");
    let wrangler_jsonc = frontend_path.join("wrangler.jsonc");

    // Read original file to verify it has comments
    let original_content = fs::read_to_string(&wrangler_jsonc)?;
    assert!(original_content.contains("/**"), "Original file should have block comments");
    assert!(original_content.contains("*/"), "Original file should have block comments");

    // Rename the project
    workspace.rename_project("test-project", "frontend", "web-app")?;

    // Verify comments are preserved in renamed project
    let new_path = project_path.join("apps").join("web-app");
    let new_wrangler_jsonc = new_path.join("wrangler.jsonc");
    let updated_content = fs::read_to_string(&new_wrangler_jsonc)?;
    
    assert!(updated_content.contains("/**"), "Renamed file should preserve block comments");
    assert!(updated_content.contains("*/"), "Renamed file should preserve block comments");
    assert!(updated_content.contains(r#""name": "web-app""#), "Should have updated name");

    Ok(())
}

#[test]
fn test_rename_fails_outside_moonflare_workspace() -> anyhow::Result<()> {
    log("→ Rename Fails Outside Moonflare Workspace");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create empty directory but don't initialize as Moonflare workspace
    let empty_dir = workspace.create_subdirectory("empty-dir")?;
    workspace.create_file_in_directory(&empty_dir, "dummy.txt", "test")?;

    let stderr = workspace.rename_project_should_fail("empty-dir", "old", "new")?;

    // Check error message
    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("not in a moonflare workspace"),
        "Error should mention not in workspace, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_rename_all_project_types_comprehensive() -> anyhow::Result<()> {
    log("→ Rename All Project Types Comprehensive Test");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;
    
    // Add all project types
    workspace.add_project("test-project", &ProjectType::React, "react-app")?;
    workspace.add_project("test-project", &ProjectType::Astro, "astro-site")?;
    workspace.add_project("test-project", &ProjectType::DurableObject, "durable-obj")?;
    workspace.add_project("test-project", &ProjectType::Crate, "rust-crate")?;

    let project_path = workspace.path().join("test-project");

    // Rename all projects
    workspace.rename_project("test-project", "react-app", "web-client")?;
    workspace.rename_project("test-project", "astro-site", "landing")?;
    workspace.rename_project("test-project", "durable-obj", "state-manager")?;
    workspace.rename_project("test-project", "rust-crate", "wasm-utils")?;

    // Verify all renames worked
    assert!(project_path.join("apps").join("web-client").is_dir(), "React renamed");
    assert!(project_path.join("sites").join("landing").is_dir(), "Astro renamed");
    assert!(project_path.join("workers").join("state-manager").is_dir(), "DO renamed");
    assert!(project_path.join("crates").join("wasm-utils").is_dir(), "Crate renamed");

    // Verify old directories are gone
    assert!(!project_path.join("apps").join("react-app").exists(), "Old React gone");
    assert!(!project_path.join("sites").join("astro-site").exists(), "Old Astro gone");
    assert!(!project_path.join("workers").join("durable-obj").exists(), "Old DO gone");
    assert!(!project_path.join("crates").join("rust-crate").exists(), "Old crate gone");

    // Verify wrangler configs were updated
    let web_client_config = fs::read_to_string(
        project_path.join("apps").join("web-client").join("wrangler.jsonc")
    )?;
    assert!(web_client_config.contains(r#""name": "web-client""#), "React wrangler updated");

    let landing_config = fs::read_to_string(
        project_path.join("sites").join("landing").join("wrangler.jsonc")
    )?;
    assert!(landing_config.contains(r#""name": "landing""#), "Astro wrangler updated");

    let state_manager_config = fs::read_to_string(
        project_path.join("workers").join("state-manager").join("wrangler.toml")
    )?;
    assert!(state_manager_config.contains(r#"name = "state-manager""#), "DO wrangler updated");

    Ok(())
}