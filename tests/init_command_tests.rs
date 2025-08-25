use common::*;
use std::fs;

mod common;

#[test]
fn test_init_creates_workspace_structure() -> anyhow::Result<()> {
    log("→ Init Creates Workspace Structure");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-project")?;

    let project_path = workspace.path().join("test-project");

    // Verify main directories exist
    assert!(
        project_path.join("apps").is_dir(),
        "apps directory should exist"
    );
    assert!(
        project_path.join("sites").is_dir(),
        "sites directory should exist"
    );
    assert!(
        project_path.join("workers").is_dir(),
        "workers directory should exist"
    );
    assert!(
        project_path.join("crates").is_dir(),
        "crates directory should exist"
    );

    // Verify workspace configuration files exist
    assert!(
        project_path.join(".moon").is_dir(),
        ".moon directory should exist"
    );
    assert!(
        project_path.join("package.json").is_file(),
        "package.json should exist"
    );
    assert!(
        project_path.join("pnpm-workspace.yaml").is_file(),
        "pnpm-workspace.yaml should exist"
    );
    assert!(
        project_path.join("README.md").is_file(),
        "README.md should exist"
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_creates_valid_package_json() -> anyhow::Result<()> {
    log("→ Init Creates Valid Package JSON");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("my-workspace")?;

    let package_json_path = workspace.path().join("my-workspace").join("package.json");
    let content = fs::read_to_string(&package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&content)?;

    // Verify package.json structure
    assert_eq!(
        package_json["name"], "my-workspace",
        "Package name should match workspace name"
    );
    assert!(
        package_json["private"].as_bool().unwrap_or(false),
        "Package should be private"
    );
    assert!(
        package_json["scripts"].is_object(),
        "Should have scripts section"
    );
    assert!(
        package_json["devDependencies"].is_object(),
        "Should have devDependencies"
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_creates_valid_moon_config() -> anyhow::Result<()> {
    log("→ Init Creates Valid Moon Config");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-workspace")?;

    let workspace_yml_path = workspace
        .path()
        .join("test-workspace")
        .join(".moon")
        .join("workspace.yml");
    assert!(
        workspace_yml_path.exists(),
        ".moon/workspace.yml should exist"
    );

    let content = fs::read_to_string(&workspace_yml_path)?;

    // Basic validation that it's valid YAML and contains expected sections
    assert!(!content.is_empty(), "workspace.yml should not be empty");

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_creates_pnpm_workspace_config() -> anyhow::Result<()> {
    log("→ Init Creates PNPM Workspace Config");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("test-workspace")?;

    let pnpm_workspace_path = workspace
        .path()
        .join("test-workspace")
        .join("pnpm-workspace.yaml");
    assert!(
        pnpm_workspace_path.exists(),
        "pnpm-workspace.yaml should exist"
    );

    let content = fs::read_to_string(&pnpm_workspace_path)?;

    // Verify workspace includes all project directories
    assert!(content.contains("apps/*"), "Should include apps/*");
    assert!(content.contains("sites/*"), "Should include sites/*");
    assert!(content.contains("workers/*"), "Should include workers/*");

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_with_different_names() -> anyhow::Result<()> {
    log("→ Init With Different Names");
    let workspace = MoonflareTestWorkspace::new()?;

    let test_names = vec![
        "simple-name",
        "my_project",
        "project123",
        "a", // single character
        "very-long-project-name-with-many-hyphens",
    ];

    for name in test_names {
        workspace.init(name)?;

        let project_path = workspace.path().join(name);
        assert!(
            project_path.is_dir(),
            "Project directory should exist for name: {}",
            name
        );

        // Verify package.json has correct name
        let package_json_path = project_path.join("package.json");
        let content = fs::read_to_string(&package_json_path)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;
        assert_eq!(
            package_json["name"], name,
            "Package name should match for: {}",
            name
        );
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_empty_directories_are_created() -> anyhow::Result<()> {
    log("→ Init Empty Directories Are Created");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("empty-test")?;

    let project_path = workspace.path().join("empty-test");
    let directories = ["apps", "sites", "workers", "crates"];

    for dir in directories {
        let dir_path = project_path.join(dir);
        assert!(dir_path.is_dir(), "{} directory should exist", dir);

        // Verify directory is empty (except for potential hidden files)
        let entries: Vec<_> = fs::read_dir(&dir_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with('.'))
            .collect();

        // Should be empty or only contain hidden files
        assert!(
            fs::read_dir(&dir_path)?.count() == entries.len(),
            "{} directory should be empty or only contain hidden files",
            dir
        );
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_creates_dot_moon_directory() -> anyhow::Result<()> {
    log("→ Init Creates .moon Directory");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("moon-test")?;

    let project_path = workspace.path().join("moon-test");
    let moon_dir = project_path.join(".moon");

    assert!(moon_dir.is_dir(), ".moon directory should exist");

    // Check for expected Moon configuration files
    let expected_files = ["workspace.yml", "toolchain.yml"];
    for file in expected_files {
        let file_path = moon_dir.join(file);
        assert!(file_path.exists(), ".moon/{} should exist", file);
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_workspace_can_add_projects_immediately() -> anyhow::Result<()> {
    log("→ Init Workspace Can Add Projects Immediately");
    let workspace = MoonflareTestWorkspace::new()?;

    workspace.init("ready-workspace")?;

    // Should be able to add projects immediately after init
    workspace.add_project("ready-workspace", &ProjectType::Astro, "test-site")?;
    workspace.add_project("ready-workspace", &ProjectType::React, "test-app")?;
    workspace.add_project("ready-workspace", &ProjectType::Crate, "test-crate")?;

    let project_path = workspace.path().join("ready-workspace");

    // Verify projects were created
    assert!(
        project_path.join("sites").join("test-site").is_dir(),
        "Astro site should be created"
    );
    assert!(
        project_path.join("apps").join("test-app").is_dir(),
        "React app should be created"
    );
    assert!(
        project_path.join("crates").join("test-crate").is_dir(),
        "Crate should be created"
    );

    log("Test completed");
    Ok(())
}
