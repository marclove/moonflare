use common::*;
use std::fs;
use std::process::Command;

mod common;

#[test]
fn test_force_flag_with_regular_project_names() -> anyhow::Result<()> {
    log("→ Force Flag with Regular Project Names");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create a directory with some content
    let project_dir = workspace.path().join("test-project");
    fs::create_dir_all(&project_dir)?;
    fs::write(project_dir.join("existing.txt"), "existing content")?;
    fs::write(project_dir.join("README.md"), "# Existing README")?;
    
    // Try init without force - should fail
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg("test-project").current_dir(workspace.path());
    
    let output = run_command_with_timeout(cmd, 5)?;
    assert!(!output.status.success(), "Init should fail in non-empty directory without force");
    
    // Try init with force - should succeed with warning
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg("test-project").arg("--force").current_dir(workspace.path());
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with --force: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Warning") && stdout.contains("--force"),
        "Output should contain warning about force flag, got: {}",
        stdout
    );
    
    // Verify workspace was created and original files preserved
    assert!(project_dir.join("apps").is_dir(), "apps directory should exist");
    assert!(project_dir.join("existing.txt").exists(), "Original file should exist");
    assert!(project_dir.join("README.md").exists(), "Original README should exist");
    assert!(project_dir.join("package.json").exists(), "package.json should be created");
    
    log("Test completed");
    Ok(())
}

#[test]
fn test_force_flag_shows_correct_file_count() -> anyhow::Result<()> {
    log("→ Force Flag Shows Correct File Count");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create a subdirectory with exactly 3 files
    let subdir = workspace.create_subdirectory("file-count-test")?;
    workspace.create_file_in_directory(&subdir, "file1.txt", "content1")?;
    workspace.create_file_in_directory(&subdir, "file2.js", "console.log('test')")?;
    workspace.create_file_in_directory(&subdir, "file3.md", "# Test")?;
    
    // Run init . --force
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").arg("--force").current_dir(&subdir);
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with --force: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("contains 3 files"),
        "Should show correct file count (3), got: {}",
        stdout
    );
    
    log("Test completed");
    Ok(())
}

#[test]
fn test_empty_directory_doesnt_show_force_warning() -> anyhow::Result<()> {
    log("→ Empty Directory Doesn't Show Force Warning");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create an empty subdirectory
    let subdir = workspace.create_subdirectory("empty-test")?;
    
    // Run init . --force in empty directory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").arg("--force").current_dir(&subdir);
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with --force in empty directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should NOT show warning about --force since directory is empty
    assert!(
        !stdout.contains("Warning"),
        "Should not show warning in empty directory, got: {}",
        stdout
    );
    
    // Should show normal success message
    assert!(
        stdout.contains("Successfully created empty-test monorepo!"),
        "Should show success message, got: {}",
        stdout
    );
    
    log("Test completed");
    Ok(())
}

#[test]
fn test_force_flag_with_hidden_files() -> anyhow::Result<()> {
    log("→ Force Flag with Hidden Files");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create a subdirectory with hidden files
    let subdir = workspace.create_subdirectory("hidden-files-test")?;
    workspace.create_file_in_directory(&subdir, ".hidden", "hidden content")?;
    workspace.create_file_in_directory(&subdir, ".gitignore", "node_modules/")?;
    workspace.create_file_in_directory(&subdir, "visible.txt", "visible content")?;
    
    // Without --force should fail
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").current_dir(&subdir);
    
    let output = run_command_with_timeout(cmd, 5)?;
    assert!(!output.status.success(), "Should fail without --force due to visible file");
    
    // With --force should succeed and count all files (including hidden)
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").arg("--force").current_dir(&subdir);
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with --force: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("contains 3 files"),
        "Should count all files including hidden ones (3 total), got: {}",
        stdout
    );
    
    // Verify all original files are preserved
    assert!(subdir.join(".hidden").exists(), "Hidden file should be preserved");
    assert!(subdir.join(".gitignore").exists(), "Gitignore should be preserved");
    assert!(subdir.join("visible.txt").exists(), "Visible file should be preserved");
    
    log("Test completed");
    Ok(())
}

#[test]
fn test_force_flag_with_subdirectories() -> anyhow::Result<()> {
    log("→ Force Flag with Subdirectories");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create a subdirectory with nested structure
    let subdir = workspace.create_subdirectory("nested-test")?;
    let nested_dir = subdir.join("existing-subdir");
    fs::create_dir_all(&nested_dir)?;
    workspace.create_file_in_directory(&nested_dir, "nested-file.txt", "nested content")?;
    workspace.create_file_in_directory(&subdir, "root-file.txt", "root content")?;
    
    // Should count directories as entries too
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init").arg(".").arg("--force").current_dir(&subdir);
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with --force: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("contains 2 files"),
        "Should count both file and directory as entries (2 total), got: {}",
        stdout
    );
    
    // Verify existing structure is preserved
    assert!(subdir.join("existing-subdir").is_dir(), "Existing subdirectory should be preserved");
    assert!(nested_dir.join("nested-file.txt").exists(), "Nested file should be preserved");
    assert!(subdir.join("root-file.txt").exists(), "Root file should be preserved");
    
    // Verify workspace was created
    assert!(subdir.join("apps").is_dir(), "apps directory should be created");
    assert!(subdir.join("package.json").exists(), "package.json should be created");
    
    log("Test completed");
    Ok(())
}

#[test]
fn test_init_dot_with_path_flag() -> anyhow::Result<()> {
    log("→ Init Dot with --path Flag (Edge Case)");
    let workspace = MoonflareTestWorkspace::new()?;
    
    // Create a subdirectory with a valid name to run the command from
    let valid_subdir = workspace.create_subdirectory("valid-workspace-name")?;
    
    // This is an edge case: using both "." and --path should work
    // It should create a workspace in the specified path named after the current directory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init")
        .arg(".")
        .arg("--path")
        .arg(workspace.path().to_string_lossy().as_ref())
        .current_dir(&valid_subdir);
    
    let output = run_command_with_timeout(cmd, 10)?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Failed to init with '.' and --path: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    // Should create workspace based on current directory name (valid-workspace-name)
    let workspace_dir = workspace.path().join("valid-workspace-name");
    assert!(workspace_dir.join("apps").is_dir(), "Workspace should be created with proper structure");
    assert!(workspace_dir.join("package.json").exists(), "package.json should exist");
    
    // Verify package.json uses correct name
    let package_json_content = fs::read_to_string(workspace_dir.join("package.json"))?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;
    assert_eq!(
        package_json["name"], 
        "valid-workspace-name", 
        "Package name should match directory name"
    );
    
    log("Test completed");
    Ok(())
}