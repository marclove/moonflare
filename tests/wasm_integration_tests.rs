use common::*;

mod common;

#[test]
fn test_crate_then_typescript() -> anyhow::Result<()> {
    log("→ Crate Then TypeScript");
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

    // Verify WASM files are copied to TypeScript project dist
    workspace.verify_wasm_files_in_typescript_dist(
        "test-workspace",
        &ProjectType::Astro,
        "dashboard",
        &["math".to_string()],
    )?;

    log("Test completed");
    Ok(())
}

#[test]
fn test_wasm_distribution_to_multiple_typescript_projects() -> anyhow::Result<()> {
    log("→ WASM Distribution to Multiple TypeScript Projects");
    let workspace = MoonflareTestWorkspace::new()?;
    workspace.init("test-workspace")?;

    // Add multiple TypeScript projects
    workspace.add_project("test-workspace", &ProjectType::Astro, "site")?;
    workspace.add_project("test-workspace", &ProjectType::React, "app")?;
    workspace.add_project("test-workspace", &ProjectType::DurableObject, "worker")?;

    // Add multiple crates
    workspace.add_project("test-workspace", &ProjectType::Crate, "math-utils")?;
    workspace.add_project("test-workspace", &ProjectType::Crate, "crypto-lib")?;

    // Build everything
    workspace.build("test-workspace")?;

    let crate_names = vec!["math-utils".to_string(), "crypto-lib".to_string()];

    // Verify WASM files exist in shared-wasm
    workspace.verify_wasm_files_exist("test-workspace", &crate_names)?;

    // Verify WASM files are distributed to ALL TypeScript projects
    workspace.verify_wasm_files_in_typescript_dist(
        "test-workspace",
        &ProjectType::Astro,
        "site",
        &crate_names,
    )?;

    workspace.verify_wasm_files_in_typescript_dist(
        "test-workspace",
        &ProjectType::React,
        "app",
        &crate_names,
    )?;

    workspace.verify_wasm_files_in_typescript_dist(
        "test-workspace",
        &ProjectType::DurableObject,
        "worker",
        &crate_names,
    )?;

    log("Test completed");
    Ok(())
}
