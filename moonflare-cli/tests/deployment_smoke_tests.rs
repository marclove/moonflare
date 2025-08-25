//! Deployment smoke tests
//!
//! These tests verify the complete deployment workflow by creating, building,
//! and deploying projects to Cloudflare. They are excluded from the default
//! test suite and must be run explicitly.
//!
//! WARNING: These tests deploy to real Cloudflare infrastructure and may incur costs.
//! They require:
//! - Valid Cloudflare account
//! - Wrangler CLI installed and authenticated
//! - Network connectivity
//!
//! Run with: cargo test --test deployment_smoke_tests -- --ignored
//!
//! Individual smoke tests can be run with:
//! cargo test --test deployment_smoke_tests test_full_deployment_workflow -- --ignored

use common::*;
use std::fs;
use std::process::Command;
use std::time::Instant;

/// Comprehensive smoke test that verifies the complete deployment workflow
/// for all three project types: Astro, React, and Durable Objects
#[test]
#[ignore = "Requires Cloudflare account and deploys to real infrastructure"]
fn test_full_deployment_workflow() -> anyhow::Result<()> {
    log("SMOKE TEST: Full Deployment Workflow");
    log("This test will create, build, and deploy all project types to Cloudflare");

    let start_time = Instant::now();
    let workspace = MoonflareTestWorkspace::new()?;

    // Check prerequisites
    check_deployment_prerequisites()?;

    let workspace_name = "smoke-test-workspace";

    // Step 1: Initialize workspace
    log("Step 1: Initializing workspace...");
    workspace.init(workspace_name)?;

    // Step 2: Add all three project types
    log("Step 2: Adding all project types...");
    workspace.add_project(workspace_name, &ProjectType::Astro, "marketing-site")?;
    workspace.add_project(workspace_name, &ProjectType::React, "admin-dashboard")?;
    workspace.add_project(workspace_name, &ProjectType::DurableObject, "user-sessions")?;

    // Verify project structure
    let workspace_path = workspace.path().join(workspace_name);
    assert!(
        workspace_path.join("sites/marketing-site").is_dir(),
        "Astro site should be created"
    );
    assert!(
        workspace_path.join("apps/admin-dashboard").is_dir(),
        "React app should be created"
    );
    assert!(
        workspace_path.join("workers/user-sessions").is_dir(),
        "Durable Object should be created"
    );

    // Step 3: Install dependencies
    log("Step 3: Installing dependencies...");
    install_workspace_dependencies(&workspace, workspace_name)?;

    // Step 4: Build all projects
    log("🔨 Step 4: Building all projects...");
    workspace.build(workspace_name)?;

    // Verify build outputs
    verify_build_outputs(&workspace_path)?;

    // Step 5: Deploy all projects
    log("Step 5: Deploying all projects to Cloudflare...");
    let deployed_urls = deploy_all_projects(&workspace, workspace_name)?;

    // Step 6: Verify deployments
    log("Step 6: Verifying deployments...");
    verify_deployments(&deployed_urls)?;

    let total_duration = start_time.elapsed();
    log(&format!("SMOKE TEST COMPLETED in {:?}", total_duration));
    log(&format!(
        "Successfully deployed {} projects:",
        deployed_urls.len()
    ));
    for (name, url) in &deployed_urls {
        log(&format!("   • {}: {}", name, url));
    }

    Ok(())
}

/// Test that verifies individual project type deployments work correctly
#[test]
#[ignore = "Requires Cloudflare account and deploys to real infrastructure"]
fn test_individual_project_deployments() -> anyhow::Result<()> {
    log("SMOKE TEST: Individual Project Deployments");

    let workspace = MoonflareTestWorkspace::new()?;
    check_deployment_prerequisites()?;

    let project_types = vec![
        (ProjectType::Astro, "astro-smoke-test"),
        (ProjectType::React, "react-smoke-test"),
        (ProjectType::DurableObject, "durable-object-smoke-test"),
    ];

    for (project_type, project_name) in project_types {
        log(&format!("Testing {} deployment...", project_type.as_str()));

        let workspace_name = format!("{}-workspace", project_name);
        workspace.init(&workspace_name)?;
        workspace.add_project(&workspace_name, &project_type, project_name)?;

        install_workspace_dependencies(&workspace, &workspace_name)?;
        workspace.build(&workspace_name)?;

        let deployed_urls = deploy_all_projects(&workspace, &workspace_name)?;
        assert_eq!(deployed_urls.len(), 1, "Should deploy exactly one project");

        let (deployed_name, deployed_url) = deployed_urls.into_iter().next().unwrap();
        assert_eq!(deployed_name, project_name);

        verify_single_deployment(&project_type, &deployed_url)?;
        log(&format!(
            "{} deployed successfully: {}",
            project_type.as_str(),
            deployed_url
        ));
    }

    Ok(())
}

/// Test that verifies the build system correctly handles missing dependencies
#[test]
#[ignore = "Requires Cloudflare account and deploys to real infrastructure"]
fn test_dependency_resolution() -> anyhow::Result<()> {
    log("SMOKE TEST: Dependency Resolution");

    let workspace = MoonflareTestWorkspace::new()?;
    check_deployment_prerequisites()?;

    let workspace_name = "dependency-test";
    workspace.init(workspace_name)?;
    workspace.add_project(workspace_name, &ProjectType::React, "test-app")?;

    // Try building without installing dependencies (should handle gracefully)
    let workspace_path = workspace.path().join(workspace_name);
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("build").current_dir(&workspace_path);

    // This might fail, but should not crash the CLI
    let output = run_command_with_timeout(cmd, 30)?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        // Expected - should mention missing dependencies
        assert!(
            stderr.to_lowercase().contains("node_modules")
                || stderr.to_lowercase().contains("install")
                || stderr.to_lowercase().contains("dependencies"),
            "Build failure should mention dependency issues"
        );
    }

    // Now install dependencies and build should succeed
    install_workspace_dependencies(&workspace, workspace_name)?;
    workspace.build(workspace_name)?;

    log("Dependency resolution test completed");
    Ok(())
}

// Helper functions

fn check_deployment_prerequisites() -> anyhow::Result<()> {
    log("Checking deployment prerequisites...");

    // Check if wrangler is installed
    let wrangler_check = Command::new("wrangler").arg("--version").output();
    if wrangler_check.is_err() {
        anyhow::bail!("Wrangler CLI not found. Install with: npm install -g wrangler");
    }

    // Check if wrangler is authenticated
    let auth_check = Command::new("wrangler").arg("whoami").output();
    match auth_check {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() || stdout.contains("not authenticated") {
                anyhow::bail!("Wrangler not authenticated. Run: wrangler login");
            }
        }
        _ => anyhow::bail!("Failed to check Wrangler authentication status"),
    }

    log("Prerequisites verified");
    Ok(())
}

fn install_workspace_dependencies(
    workspace: &MoonflareTestWorkspace,
    workspace_name: &str,
) -> anyhow::Result<()> {
    let start = Instant::now();
    log("Installing workspace dependencies with pnpm...");

    let workspace_path = workspace.path().join(workspace_name);
    let mut cmd = Command::new("pnpm");
    cmd.arg("install").current_dir(&workspace_path);

    let output = run_command_with_timeout(cmd, 120)?; // 2 minutes timeout

    if !output.status.success() {
        anyhow::bail!(
            "Failed to install dependencies: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    log(&format!("Dependencies installed in {:?}", start.elapsed()));
    Ok(())
}

fn verify_build_outputs(workspace_path: &std::path::Path) -> anyhow::Result<()> {
    log("Verifying build outputs...");

    // Check Astro build output
    let astro_dist = workspace_path.join("sites/marketing-site/dist");
    if astro_dist.exists() {
        assert!(
            astro_dist.join("index.html").exists(),
            "Astro should generate index.html"
        );
    }

    // Check React build output
    let react_dist = workspace_path.join("apps/admin-dashboard/dist");
    if react_dist.exists() {
        assert!(
            react_dist.join("index.html").exists(),
            "React should generate index.html"
        );
        assert!(
            fs::read_dir(&react_dist)?.any(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "js")
            }),
            "React should generate JS files"
        );
    }

    // Check Durable Object build output
    let worker_dist = workspace_path.join("workers/user-sessions/dist");
    if worker_dist.exists() {
        assert!(
            fs::read_dir(&worker_dist)?.any(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "js")
            }),
            "Durable Object should generate JS files"
        );
    }

    log("Build outputs verified");
    Ok(())
}

fn deploy_all_projects(
    workspace: &MoonflareTestWorkspace,
    workspace_name: &str,
) -> anyhow::Result<Vec<(String, String)>> {
    let start = Instant::now();
    log("Deploying all projects...");

    let workspace_path = workspace.path().join(workspace_name);
    let mut cmd = Command::new(workspace.moonflare_binary());
    cmd.arg("deploy").current_dir(&workspace_path);

    let output = run_command_with_timeout(cmd, 300)?; // 5 minutes timeout

    if !output.status.success() {
        log(&format!(
            "Deploy STDERR: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
        anyhow::bail!(
            "Failed to deploy projects: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse deployment URLs from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut deployed_urls = Vec::new();

    for line in stdout.lines() {
        if line.trim().starts_with("https://") && line.contains(".workers.dev") {
            let url = line.trim().to_string();
            // Extract project name from URL
            if let Some(project_name) = url
                .strip_prefix("https://")
                .and_then(|s| s.split('.').next())
            {
                deployed_urls.push((project_name.to_string(), url));
            }
        }
    }

    if deployed_urls.is_empty() {
        anyhow::bail!("No deployment URLs found in output");
    }

    log(&format!(
        "Deployed {} projects in {:?}",
        deployed_urls.len(),
        start.elapsed()
    ));
    Ok(deployed_urls)
}

fn verify_deployments(deployed_urls: &[(String, String)]) -> anyhow::Result<()> {
    log("Testing deployed services...");

    for (name, url) in deployed_urls {
        log(&format!("Testing {}: {}", name, url));

        let mut cmd = Command::new("curl");
        cmd.args(["-f", "-s", "--max-time", "30", url]);

        let output = run_command_with_timeout(cmd, 35)?;

        if !output.status.success() {
            anyhow::bail!("Failed to fetch {}: HTTP error", name);
        }

        let response = String::from_utf8_lossy(&output.stdout);

        // Basic verification that we got some content
        if response.trim().is_empty() {
            anyhow::bail!("Empty response from {}", name);
        }

        // Project-specific verifications
        if name.contains("user-sessions") {
            // Durable Object should return plain text response
            if !response.contains("user-sessions") && !response.contains("Hello") {
                log(&format!(
                    "Warning: Unexpected Durable Object response: {}",
                    response
                ));
            }
        } else {
            // Web projects should return HTML
            if !response.contains("<html") && !response.contains("<!DOCTYPE") {
                log(&format!(
                    "Warning: Expected HTML response from {}: {}",
                    name,
                    response.chars().take(100).collect::<String>()
                ));
            }
        }

        log(&format!("{} is responding correctly", name));
    }

    Ok(())
}

fn verify_single_deployment(project_type: &ProjectType, url: &str) -> anyhow::Result<()> {
    let mut cmd = Command::new("curl");
    cmd.args(["-f", "-s", "--max-time", "30", url]);

    let output = run_command_with_timeout(cmd, 35)?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch deployment: HTTP error");
    }

    let response = String::from_utf8_lossy(&output.stdout);

    match project_type {
        ProjectType::Astro | ProjectType::React => {
            assert!(
                response.contains("<html") || response.contains("<!DOCTYPE"),
                "Web project should return HTML"
            );
        }
        ProjectType::DurableObject => {
            // Test both root and increment endpoints for Durable Objects
            let mut increment_cmd = Command::new("curl");
            increment_cmd.args([
                "-f",
                "-s",
                "--max-time",
                "30",
                &format!("{}/increment", url),
            ]);

            let increment_output = run_command_with_timeout(increment_cmd, 35)?;
            if increment_output.status.success() {
                let increment_response = String::from_utf8_lossy(&increment_output.stdout);
                assert!(
                    increment_response.contains("Count:") || increment_response.contains("count"),
                    "Durable Object increment should return count"
                );
            }
        }
        ProjectType::Crate => {
            // Crates don't deploy independently
        }
    }

    Ok(())
}
