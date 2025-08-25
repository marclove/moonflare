use common::*;
use proptest::prelude::*;

mod common;

proptest! {
    #![proptest_config(ProptestConfig { cases: 3, max_global_rejects: 1000, ..ProptestConfig::default() })]
    #[test]
    fn test_property_based_project_sequences(
        projects in arb_project_sequence()
    ) {
        log(&format!("→ Project Sequences ({} projects)", projects.len()));
        let workspace = MoonflareTestWorkspace::new().unwrap();
        workspace.init("test-workspace").unwrap();

        let mut crates_added = Vec::new();
        let mut typescript_projects = Vec::new();

        // Add all projects in sequence
        for project in &projects {
            workspace.add_project("test-workspace", &project.project_type, &project.name).unwrap();

            if project.project_type.is_crate() {
                crates_added.push(project.name.clone());
            } else if project.project_type.is_typescript() {
                typescript_projects.push(project.clone());
            }
        }

        // Verify final state
        let should_have_wasm_deps = !crates_added.is_empty();

        for ts_project in &typescript_projects {
            workspace.verify_typescript_project_has_wasm_deps(
                "test-workspace",
                &ts_project.project_type,
                &ts_project.name,
                should_have_wasm_deps
            ).unwrap();
        }

        if !crates_added.is_empty() {
            workspace.verify_shared_wasm_has_crate_deps("test-workspace", &crates_added).unwrap();
        }

        // Build should always succeed
        workspace.build("test-workspace").unwrap();

        // If we have crates AND TypeScript projects, verify WASM files exist
        if !crates_added.is_empty() && !typescript_projects.is_empty() {
            workspace.verify_wasm_files_exist("test-workspace", &crates_added).unwrap();

            // Also verify WASM files are in each TypeScript project's dist
            for ts_project in &typescript_projects {
                workspace.verify_wasm_files_in_typescript_dist(
                    "test-workspace",
                    &ts_project.project_type,
                    &ts_project.name,
                    &crates_added
                ).unwrap();
            }
        }

        log("Property test completed");
    }
}