use super::{ProjectAdd, ProjectType};
use proptest::prelude::*;

// Property test generators
prop_compose! {
    pub fn arb_project_type()(project_type in prop_oneof![
        Just(ProjectType::Astro),
        Just(ProjectType::React),
        Just(ProjectType::DurableObject),
        Just(ProjectType::Crate),
    ]) -> ProjectType {
        project_type
    }
}

prop_compose! {
    pub fn arb_project_name()(name in "[a-z][a-z0-9]*(-[a-z0-9]+)*") -> String {
        // Ensure the name is between 2-15 characters and doesn't end with a hyphen
        if name.len() < 2 {
            format!("{}lib", name)
        } else if name.len() > 15 {
            name[..15].trim_end_matches('-').to_string()
        } else {
            name.trim_end_matches('-').to_string()
        }
    }
}

prop_compose! {
    pub fn arb_project_add()(
        project_type in arb_project_type(),
        name in arb_project_name()
    ) -> ProjectAdd {
        ProjectAdd { project_type, name }
    }
}

prop_compose! {
    pub fn arb_project_sequence()(
        projects in prop::collection::vec(arb_project_add(), 1..5)
    ) -> Vec<ProjectAdd> {
        // Ensure unique names
        let mut seen_names = std::collections::HashSet::new();
        let mut unique_projects = Vec::new();

        for mut project in projects {
            let mut counter = 1;
            let base_name = project.name.clone();

            while seen_names.contains(&project.name) {
                // Use hyphen separator to maintain valid crate naming
                project.name = format!("{}-{}", base_name, counter);
                counter += 1;
            }

            seen_names.insert(project.name.clone());
            unique_projects.push(project);
        }

        unique_projects
    }
}
