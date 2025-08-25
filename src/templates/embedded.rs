// Embedded template content for project types

pub const ASTRO_TEMPLATE: &str = include_str!("astro.template");
pub const REACT_TEMPLATE: &str = include_str!("react.template");
pub const DURABLE_OBJECT_TEMPLATE: &str = include_str!("durable-object.template");
pub const CRATE_TEMPLATE: &str = include_str!("crate.template");
pub const WORKSPACE_TEMPLATE: &str = include_str!("workspace.template");

pub fn get_template(project_type: &str) -> Option<&'static str> {
    match project_type {
        "astro" => Some(ASTRO_TEMPLATE),
        "react" => Some(REACT_TEMPLATE),
        "durable-object" | "worker" => Some(DURABLE_OBJECT_TEMPLATE),
        "crate" => Some(CRATE_TEMPLATE),
        "workspace" => Some(WORKSPACE_TEMPLATE),
        _ => None,
    }
}
