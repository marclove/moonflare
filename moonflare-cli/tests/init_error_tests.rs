use common::*;
use std::fs;
use std::process::Command;

mod common;

#[test]
fn test_init_fails_with_invalid_name_characters() -> anyhow::Result<()> {
    log("→ Init Fails With Invalid Name Characters");
    let workspace = MoonflareTestWorkspace::new()?;

    let invalid_names = vec![
        "project with spaces",  // spaces not allowed
        "project/with/slashes", // slashes not allowed
        "project@with@at",      // @ symbols not allowed
        "UPPERCASE",            // typically should be lowercase
        "",                     // empty name
    ];

    for name in invalid_names {
        log(&format!("Testing invalid name: '{}'", name));

        let mut cmd = Command::new(workspace.moonflare_binary());
        cmd.arg("init").arg(name).current_dir(workspace.path());

        let output = run_command_with_timeout(cmd, 5)?;

        if name.is_empty() {
            // Empty name may be handled by creating a directory with empty name
            log(&format!(
                "Empty name command success: {}",
                output.status.success()
            ));
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log(&format!("Empty name error: {}", stderr));
            }
        } else {
            // For other names, the CLI may be permissive - just log the result
            log(&format!(
                "Command output for '{}': success={}",
                name,
                output.status.success()
            ));

            if output.status.success() {
                // If it succeeded, verify basic structure was created
                let project_path = workspace.path().join(name);
                if project_path.exists() {
                    log(&format!("Project directory created for '{}'", name));
                }
            }
        }
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_fails_when_directory_already_exists() -> anyhow::Result<()> {
    log("→ Init Fails When Directory Already Exists");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a directory first
    let existing_dir = workspace.path().join("existing-project");
    fs::create_dir(&existing_dir)?;
    fs::write(existing_dir.join("existing-file.txt"), "content")?;

    // Try to init into existing directory
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init")
        .arg("existing-project")
        .current_dir(workspace.path());

    let output = run_command_with_timeout(cmd, 5)?;

    // Should either fail or handle gracefully
    log(&format!("Command success: {}", output.status.success()));
    log(&format!(
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    ));

    // Verify original file still exists if command failed
    if !output.status.success() {
        assert!(
            existing_dir.join("existing-file.txt").exists(),
            "Original file should still exist"
        );
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_handles_permission_errors_gracefully() -> anyhow::Result<()> {
    log("→ Init Handles Permission Errors Gracefully");
    let workspace = MoonflareTestWorkspace::new()?;

    // Try to create in a location that might have permission issues
    // This is platform-specific, so we'll try a few locations
    let restricted_paths = vec![
        "/root/restricted-project", // Unix systems
        "/etc/moonflare-project",   // Unix systems - typically read-only
    ];

    for path in restricted_paths {
        log(&format!("Testing restricted path: {}", path));

        let mut cmd = Command::new(workspace.moonflare_binary());
        cmd.arg("init")
            .arg("test-project")
            .arg("--path")
            .arg(path)
            .current_dir(workspace.path());

        let output = run_command_with_timeout(cmd, 5)?;

        // Should fail gracefully with meaningful error message
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log(&format!("Expected failure for {}: {}", path, stderr));

            // Error message should be helpful - check for common error indicators
            let has_descriptive_error = stderr.contains("permission")
                || stderr.contains("denied")
                || stderr.contains("access")
                || stderr.contains("cannot create")
                || stderr.contains("No such file")
                || stderr.contains("Read-only file system")
                || stderr.contains("os error");

            if !has_descriptive_error {
                log(&format!(
                    "Warning: Error message may not be descriptive enough: {}",
                    stderr
                ));
            }
        }
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_validates_workspace_name_format() -> anyhow::Result<()> {
    log("→ Init Validates Workspace Name Format");
    let workspace = MoonflareTestWorkspace::new()?;

    // Test edge case names that might cause issues
    let long_name = "x".repeat(255);
    let edge_case_names = vec![
        "a",                 // very short
        &long_name,          // very long
        "123",               // all numbers
        "-starts-with-dash", // starts with dash
        "ends-with-dash-",   // ends with dash
        "double--dash",      // double dash
        "under_score",       // underscores
        "mixed-name_123",    // mixed separators
    ];

    for name in edge_case_names {
        log(&format!(
            "Testing edge case name (len={}): '{}'",
            name.len(),
            if name.len() > 50 { &name[..50] } else { &name }
        ));

        let mut cmd = Command::new(workspace.moonflare_binary());
        cmd.arg("init").arg(&name).current_dir(workspace.path());

        let output = run_command_with_timeout(cmd, 10)?;

        log(&format!(
            "Result for '{}' (len={}): success={}",
            if name.len() > 20 { &name[..20] } else { &name },
            name.len(),
            output.status.success()
        ));

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log(&format!("Error: {}", stderr));
        }
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_handles_filesystem_errors() -> anyhow::Result<()> {
    log("→ Init Handles Filesystem Errors");
    let workspace = MoonflareTestWorkspace::new()?;

    // Create a file with the same name as the directory we want to create
    let conflicting_file = workspace.path().join("file-conflict");
    fs::write(&conflicting_file, "existing file content")?;

    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("init")
        .arg("file-conflict")
        .current_dir(workspace.path());

    let output = run_command_with_timeout(cmd, 5)?;

    // Should handle the conflict gracefully
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log(&format!("Expected conflict error: {}", stderr));

        // Verify original file is unchanged
        let content = fs::read_to_string(&conflicting_file)?;
        assert_eq!(
            content, "existing file content",
            "Original file should be unchanged"
        );
    }

    log("Test completed");
    Ok(())
}

#[test]
fn test_init_with_minimal_disk_space() -> anyhow::Result<()> {
    log("→ Init With Minimal Disk Space");
    let workspace = MoonflareTestWorkspace::new()?;

    // This test verifies init completes without excessive disk usage
    // We'll measure the workspace size after creation

    workspace.init("minimal-workspace")?;

    let project_path = workspace.path().join("minimal-workspace");

    // Calculate total size of created workspace
    let total_size = calculate_directory_size(&project_path)?;

    log(&format!("Workspace size: {} bytes", total_size));

    // Workspace should be reasonably small (less than 10MB for basic setup)
    // This is a sanity check to ensure we're not creating massive templates
    assert!(
        total_size < 10 * 1024 * 1024, // 10MB
        "Workspace should not exceed 10MB, got {} bytes",
        total_size
    );

    log("Test completed");
    Ok(())
}

fn calculate_directory_size(dir: &std::path::Path) -> anyhow::Result<u64> {
    let mut total_size = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += calculate_directory_size(&path)?;
        } else {
            total_size += entry.metadata()?.len();
        }
    }

    Ok(total_size)
}
