use common::*;

mod common;

#[test]
fn test_typescript_project_without_crates() -> anyhow::Result<()> {
    log("→ TypeScript Project Without Crates");
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
    log("Test completed");

    Ok(())
}

#[test]
fn test_typescript_after_crates_exist() -> anyhow::Result<()> {
    log("→ TypeScript After Crates Exist");
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

    // Build and verify WASM files are distributed to TypeScript projects
    workspace.build("test-workspace")?;
    workspace.verify_wasm_files_exist("test-workspace", &["utils".to_string()])?;
    workspace.verify_wasm_files_in_typescript_dist(
        "test-workspace",
        &ProjectType::DurableObject,
        "api",
        &["utils".to_string()],
    )?;

    log("Test completed");
    Ok(())
}
