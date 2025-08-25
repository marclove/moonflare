use common::*;
use std::fs;
use std::process::Command;

mod common;

#[test]
fn test_init_dot_in_empty_directory() -> anyhow::Result<()> {
    log("→ Init with '.' in Empty Directory");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a subdirectory with a valid workspace name
    let subdir = workspace.create_subdirectory("test-workspace")?;

    // Run moonflare init . from within the subdirectory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").current_dir(&subdir);

    let output = run_command_with_timeout(cmd, 10)?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with '.': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Verify workspace structure was created in the current directory
    assert!(subdir.join("apps").is_dir(), "apps directory should exist");
    assert!(
        subdir.join("sites").is_dir(),
        "sites directory should exist"
    );
    assert!(
        subdir.join("workers").is_dir(),
        "workers directory should exist"
    );
    assert!(
        subdir.join("crates").is_dir(),
        "crates directory should exist"
    );
    assert!(
        subdir.join(".moon").is_dir(),
        ".moon directory should exist"
    );
    assert!(
        subdir.join("package.json").is_file(),
        "package.json should exist"
    );

    // Verify package.json uses directory name as workspace name
    let package_json_content = fs::read_to_string(subdir.join("package.json"))?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;
    assert_eq!(
        package_json["name"], "test-workspace",
        "Package name should match directory name"
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_dot_in_non_empty_directory_fails() -> anyhow::Result<()> {
    log("→ Init with '.' in Non-Empty Directory Fails");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a subdirectory with existing files
    let subdir = workspace.create_subdirectory("existing-project")?;
    workspace.create_file_in_directory(&subdir, "existing-file.txt", "some content")?;
    workspace.create_file_in_directory(&subdir, "README.md", "# Existing project")?;

    // Run moonflare init . from within the non-empty subdirectory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").current_dir(&subdir);

    let output = run_command_with_timeout(cmd, 5)?;

    // Should fail because directory is not empty
    assert!(
        !output.status.success(),
        "Init should fail in non-empty directory"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("directory is not empty") || stderr_lower.contains("not empty"),
        "Error should mention non-empty directory, got: {}",
        stderr
    );

    // Verify original files still exist and no workspace files were created
    assert!(
        subdir.join("existing-file.txt").exists(),
        "Original file should still exist"
    );
    assert!(
        subdir.join("README.md").exists(),
        "Original README should still exist"
    );
    assert!(
        !subdir.join(".moon").exists(),
        ".moon directory should not be created"
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_dot_with_force_in_non_empty_directory() -> anyhow::Result<()> {
    log("→ Init with '.' --force in Non-Empty Directory");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a subdirectory with existing files
    let subdir = workspace.create_subdirectory("force-test-workspace")?;
    workspace.create_file_in_directory(&subdir, "existing.txt", "content")?;
    workspace.create_file_in_directory(&subdir, "another.js", "console.log('test')")?;

    // Run moonflare init . --force from within the non-empty subdirectory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").arg("--force").current_dir(&subdir);

    let output = run_command_with_timeout(cmd, 10)?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with '.' --force: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Warning") && stdout.contains("--force"),
        "Output should contain warning about force flag, got: {}",
        stdout
    );

    // Verify workspace structure was created
    assert!(subdir.join("apps").is_dir(), "apps directory should exist");
    assert!(
        subdir.join("sites").is_dir(),
        "sites directory should exist"
    );
    assert!(
        subdir.join("workers").is_dir(),
        "workers directory should exist"
    );
    assert!(
        subdir.join("crates").is_dir(),
        "crates directory should exist"
    );
    assert!(
        subdir.join(".moon").is_dir(),
        ".moon directory should exist"
    );
    assert!(
        subdir.join("package.json").is_file(),
        "package.json should exist"
    );

    // Verify original files still exist
    assert!(
        subdir.join("existing.txt").exists(),
        "Original file should still exist"
    );
    assert!(
        subdir.join("another.js").exists(),
        "Original JS file should still exist"
    );

    // Verify package.json uses directory name as workspace name
    let package_json_content = fs::read_to_string(subdir.join("package.json"))?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;
    assert_eq!(
        package_json["name"], "force-test-workspace",
        "Package name should match directory name"
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_regular_name_with_force_flag() -> anyhow::Result<()> {
    log("→ Init Regular Name with --force Flag");
    let workspace = MoonflareTestWorkspace::new()?;

    // First create a regular workspace
    workspace.init("test-workspace")?;

    // Try to init again with --force (should succeed and show warning)
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init")
        .arg("test-workspace")
        .arg("--force")
        .current_dir(workspace.path());

    let output = run_command_with_timeout(cmd, 10)?;

    if !output.status.success() {
        anyhow::bail!(
            "Init with --force should succeed in non-empty directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Warning") && stdout.contains("--force"),
        "Should show warning about --force flag, got: {}",
        stdout
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_dot_with_invalid_directory_name() -> anyhow::Result<()> {
    log("→ Init with '.' in Directory with Invalid Name");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a subdirectory with an invalid workspace name (contains uppercase and spaces)
    let subdir = workspace.create_subdirectory("Invalid Name With Spaces")?;

    // Run moonflare init . from within the invalidly named subdirectory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").current_dir(&subdir);

    let output = run_command_with_timeout(cmd, 5)?;

    // Should fail because directory name is not a valid workspace name
    assert!(
        !output.status.success(),
        "Init should fail with invalid directory name"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("Invalid"),
        "Error should mention invalid name, got: {}",
        stderr
    );

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_dot_creates_correct_output_messages() -> anyhow::Result<()> {
    log("→ Init with '.' Creates Correct Output Messages");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a subdirectory
    let subdir = workspace.create_subdirectory("message-test")?;

    // Run moonflare init . from within the subdirectory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").current_dir(&subdir);

    let output = run_command_with_timeout(cmd, 10)?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with '.': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show success message with directory name
    assert!(
        stdout.contains("Successfully created message-test monorepo!"),
        "Should contain success message with directory name, got: {}",
        stdout
    );

    // Should NOT contain "cd message-test" since we're already in the directory
    assert!(
        !stdout.contains("cd message-test"),
        "Should not suggest cd since we're already in the directory, got: {}",
        stdout
    );

    // Should contain the other standard next steps
    assert!(
        stdout.contains("moonflare add <type> <name>"),
        "Should contain add command suggestion, got: {}",
        stdout
    );

    log("Test completed");
    Ok(())
}
