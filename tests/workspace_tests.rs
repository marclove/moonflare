use common::*;

mod common;

#[test]
fn test_basic_workspace_creation() -> anyhow::Result<()> {
    log("→ Basic Workspace Creation");
    let workspace = MoonflareTestWorkspace::new()?;
    workspace.init("test-workspace")?;

    log("Verifying workspace structure");
    // Verify workspace structure exists
    let workspace_path = workspace.path().join("test-workspace");

    let moon_dir = workspace_path.join(".moon");
    if !moon_dir.exists() {
        anyhow::bail!(".moon directory missing at {:?}", moon_dir);
    }

    let package_json = workspace_path.join("package.json");
    if !package_json.exists() {
        anyhow::bail!("package.json missing at {:?}", package_json);
    }

    let shared_wasm = workspace_path.join("shared-wasm");
    if !shared_wasm.exists() {
        anyhow::bail!("shared-wasm directory missing at {:?}", shared_wasm);
    }

    log("All workspace structure verified");
    Ok(())
}
