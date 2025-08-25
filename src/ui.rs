use iocraft::prelude::*;
use starbase_console::ui::*;
use starbase_console::{Console, ConsoleError, EmptyReporter};

pub struct MoonflareUI {
    console: Console<EmptyReporter>,
}

impl MoonflareUI {
    pub fn new() -> Self {
        Self {
            console: Console::new(false),
        }
    }

    pub fn render_header(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<(), ConsoleError> {
        match description {
            Some(desc) => self.console.render(element! {
                Container {
                    Notice(variant: Variant::Info, title: title.to_owned()) {
                        Text(content: desc)
                    }
                }
            }),
            None => self.console.render(element! {
                Container {
                    Notice(variant: Variant::Info, title: title.to_owned())
                }
            }),
        }
    }

    pub fn render_success(&self, message: &str) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Notice(variant: Variant::Success, no_title: true) {
                Text(content: format!("✓ {}", message))
            }
        })
    }

    pub fn render_section_start(&self, title: &str) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Section(title: title.to_owned())
        })
    }

    pub fn render_project_types(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Section(title: "Available project types") {
                List {
                    ListItem {
                        Entry(name: "astro") {
                            Text(content: "Astro static site")
                        }
                    }
                    ListItem {
                        Entry(name: "react") {
                            Text(content: "React application")
                        }
                    }
                    ListItem {
                        Entry(name: "durable-object") {
                            Text(content: "Cloudflare Durable Object")
                        }
                    }
                    ListItem {
                        Entry(name: "crate") {
                            Text(content: "Rust WASM library")
                        }
                    }
                }
            }
        })
    }

    pub fn render_next_steps(&self, steps: Vec<&str>) -> Result<(), ConsoleError> {
        if steps.len() == 1 {
            // Single step - no numbering needed
            self.console.render(element! {
                Section(title: "Next step") {
                    List {
                        ListItem {
                            StyledText(content: steps[0], style: Style::Shell)
                        }
                    }
                }
            })
        } else {
            // Multiple steps - show as numbered sequence
            self.console.render(element! {
                Section(title: "Next steps") {
                    List {
                        #(steps.into_iter().enumerate().map(|(i, step)| {
                            element! {
                                ListItem {
                                    Entry(name: format!("{}.", i + 1)) {
                                        StyledText(content: step, style: Style::Shell)
                                    }
                                }
                            }
                        }))
                    }
                }
            })
        }
    }

    pub fn render_next_alternatives(&self, alternatives: Vec<&str>) -> Result<(), ConsoleError> {
        // For alternatives, don't number them - just show as options
        let title = if alternatives.len() == 1 {
            "Next step"
        } else {
            "Next steps (choose one)"
        };

        self.console.render(element! {
            Section(title: title) {
                List {
                    #(alternatives.into_iter().map(|step| {
                        element! {
                            ListItem {
                                StyledText(content: step, style: Style::Shell)
                            }
                        }
                    }))
                }
            }
        })
    }

    pub fn render_next_steps_for_project(
        &self,
        project_name: &str,
        project_type: &str,
    ) -> Result<(), ConsoleError> {
        match project_type {
            "astro" | "react" | "durable-object" => {
                let dev_cmd = format!("moonflare dev {}", project_name);
                let build_cmd = format!("moonflare build {}", project_name);
                let deploy_cmd = format!("moonflare deploy {}", project_name);
                let alternatives = vec![dev_cmd.as_str(), build_cmd.as_str(), deploy_cmd.as_str()];
                self.render_next_alternatives(alternatives)
            }
            "crate" => {
                let build_cmd = format!("moonflare build {}", project_name);
                let alternatives = vec![
                    build_cmd.as_str(),
                    "moonflare build  # Build all projects to generate WASM",
                ];
                self.render_next_alternatives(alternatives)
            }
            _ => Ok(()),
        }
    }

    pub fn render_workspace_structure(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Section(title: "Workspace structure") {
                List {
                    ListItem { Text(content: "apps/         → React applications") }
                    ListItem { Text(content: "sites/        → Astro static sites") }
                    ListItem { Text(content: "workers/      → Cloudflare Workers & Durable Objects") }
                    ListItem { Text(content: "crates/       → Rust WASM libraries") }
                    ListItem { Text(content: ".moon/        → Moon configuration") }
                }
            }
        })
    }

    pub fn render_error(
        &self,
        title: &str,
        message: &str,
        suggestions: Vec<&str>,
    ) -> Result<(), ConsoleError> {
        // Render error messages to stderr
        self.console.stderr().render(
            element! {
                Container {
                    Notice(variant: Variant::Failure, title: title.to_owned()) {
                        Text(content: message)
                    }

                    Section(title: "Solutions") {
                        List {
                            #(suggestions.into_iter().map(|suggestion| {
                                element! {
                                    ListItem {
                                        StyledText(content: suggestion, style: Style::Shell)
                                    }
                                }
                            }))
                        }
                    }
                }
            },
            self.console.theme(),
        )
    }

    pub fn render_main_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "Moonflare: Supersonic Cloudflare monorepo".to_owned()) {
                    Text(content: "A CLI utility for managing Cloudflare-focused monorepos with Moon build system.\nUses Moonrepo (https://moonrepo.dev) for task orchestration, caching, and dependency management.")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare <COMMAND>", style: Style::Shell)
                }

                Section(title: "Commands") {
                    List {
                        ListItem {
                            Entry(name: "init") {
                                Text(content: "Initialize a new Cloudflare monorepo")
                            }
                        }
                        ListItem {
                            Entry(name: "add") {
                                Text(content: "Add a new project to the monorepo")
                            }
                        }
                        ListItem {
                            Entry(name: "build") {
                                Text(content: "Build project(s) - all projects or specify one")
                            }
                        }
                        ListItem {
                            Entry(name: "dev") {
                                Text(content: "Start development server - all projects or specify one")
                            }
                        }
                        ListItem {
                            Entry(name: "deploy") {
                                Text(content: "Deploy project(s) to Cloudflare - all projects or specify one")
                            }
                        }
                        ListItem {
                            Entry(name: "help") {
                                Text(content: "Print this message or help for a specific command")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                        ListItem {
                            Entry(name: "-V, --version") {
                                Text(content: "Print version")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare init my-app              # Create new monorepo", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare add react frontend       # Add React app", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare build                    # Build all projects", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare dev frontend             # Start dev server for one project", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare deploy                   # Deploy all projects", style: Style::Shell)
                        }
                    }
                }

                Section(title: "About Moon Integration") {
                    List {
                        ListItem {
                            Text(content: "Moonflare uses Moon (https://moonrepo.dev) for task orchestration and caching")
                        }
                        ListItem {
                            Text(content: "Advanced workflows: use 'moon run <target>' for custom task execution")
                        }
                        ListItem {
                            Text(content: "Task dependencies ensure WASM crates build before TypeScript projects")
                        }
                    }
                }
            }
        })
    }

    pub fn render_init_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "moonflare init".to_owned()) {
                    Text(content: "Initialize a new Cloudflare monorepo")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare init [OPTIONS] <NAME>", style: Style::Shell)
                }

                Section(title: "Arguments") {
                    List {
                        ListItem {
                            Entry(name: "<NAME>") {
                                Text(content: "Name of the monorepo (use '.' for current directory)")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "--path <PATH>") {
                                Text(content: "Directory to create the monorepo in")
                            }
                        }
                        ListItem {
                            Entry(name: "--force") {
                                Text(content: "Force initialization in non-empty directories")
                            }
                        }
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare init my-app              # Create in ./my-app/", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare init .                   # Create in current directory", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare init . --force           # Create in non-empty directory", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare init app --path ~/code   # Create in ~/code/app/", style: Style::Shell)
                        }
                    }
                }
            }
        })
    }

    pub fn render_add_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "moonflare add".to_owned()) {
                    Text(content: "Add a new project to the monorepo")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare add <TYPE> <NAME>", style: Style::Shell)
                }

                Section(title: "Arguments") {
                    List {
                        ListItem {
                            Entry(name: "<TYPE>") {
                                Text(content: "Type of project (astro, react, durable-object, crate)")
                            }
                        }
                        ListItem {
                            Entry(name: "<NAME>") {
                                Text(content: "Name of the project")
                            }
                        }
                    }
                }

                Section(title: "Project Types") {
                    List {
                        ListItem {
                            Entry(name: "astro") {
                                Text(content: "Static site with Astro framework")
                            }
                        }
                        ListItem {
                            Entry(name: "react") {
                                Text(content: "React application with Vite")
                            }
                        }
                        ListItem {
                            Entry(name: "durable-object") {
                                Text(content: "Cloudflare Worker with Durable Objects")
                            }
                        }
                        ListItem {
                            Entry(name: "crate") {
                                Text(content: "Rust library compiled to WASM")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare add react frontend       # Add React app", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare add astro marketing      # Add Astro site", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare add crate utils          # Add Rust WASM crate", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare add durable-object api   # Add DO worker", style: Style::Shell)
                        }
                    }
                }
            }
        })
    }

    pub fn render_build_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "moonflare build".to_owned()) {
                    Text(content: "Build project(s) using Moon's task orchestration")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare build [PROJECT]", style: Style::Shell)
                }

                Section(title: "Arguments") {
                    List {
                        ListItem {
                            Entry(name: "[PROJECT]") {
                                Text(content: "Specific project to build (optional - builds all projects if omitted)")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare build                    # Build all projects in monorepo", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare build frontend           # Build only the 'frontend' project", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare build my-crate           # Build Rust crate (compiles to WASM)", style: Style::Shell)
                        }
                    }
                }

                Section(title: "Build Process") {
                    List {
                        ListItem {
                            Text(content: "Rust crates compile to WASM and are collected in shared-wasm/")
                        }
                        ListItem {
                            Text(content: "TypeScript projects automatically get access to built WASM modules")
                        }
                        ListItem {
                            Text(content: "Moon ensures proper build order based on project dependencies")
                        }
                    }
                }
            }
        })
    }

    pub fn render_dev_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "moonflare dev".to_owned()) {
                    Text(content: "Start development server(s) with hot reloading")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare dev [PROJECT]", style: Style::Shell)
                }

                Section(title: "Arguments") {
                    List {
                        ListItem {
                            Entry(name: "[PROJECT]") {
                                Text(content: "Specific project to run (optional - runs all dev servers if omitted)")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare dev                      # Start all dev servers", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare dev frontend             # Start dev server for 'frontend' only", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare dev marketing            # Start Astro dev server", style: Style::Shell)
                        }
                    }
                }

                Section(title: "Development Features") {
                    List {
                        ListItem {
                            Text(content: "Hot reloading for TypeScript/React/Astro projects")
                        }
                        ListItem {
                            Text(content: "Automatic WASM rebuilding when Rust crates change")
                        }
                        ListItem {
                            Text(content: "Multiple dev servers can run simultaneously on different ports")
                        }
                    }
                }
            }
        })
    }

    pub fn render_deploy_help(&self) -> Result<(), ConsoleError> {
        self.console.render(element! {
            Container {
                Notice(variant: Variant::Info, title: "moonflare deploy".to_owned()) {
                    Text(content: "Deploy project(s) to Cloudflare using Wrangler")
                }

                Section(title: "Usage") {
                    StyledText(content: "moonflare deploy [OPTIONS] [PROJECT]", style: Style::Shell)
                }

                Section(title: "Arguments") {
                    List {
                        ListItem {
                            Entry(name: "[PROJECT]") {
                                Text(content: "Specific project to deploy (optional - deploys all projects if omitted)")
                            }
                        }
                    }
                }

                Section(title: "Options") {
                    List {
                        ListItem {
                            Entry(name: "--env <ENV>") {
                                Text(content: "Environment to deploy to (e.g., staging, production)")
                            }
                        }
                        ListItem {
                            Entry(name: "-h, --help") {
                                Text(content: "Print help")
                            }
                        }
                    }
                }

                Section(title: "Examples") {
                    List {
                        ListItem {
                            StyledText(content: "moonflare deploy                   # Deploy all projects", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare deploy api               # Deploy only the 'api' worker", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare deploy --env staging     # Deploy all to staging environment", style: Style::Shell)
                        }
                        ListItem {
                            StyledText(content: "moonflare deploy site --env prod   # Deploy 'site' to production", style: Style::Shell)
                        }
                    }
                }

                Section(title: "Deployment Process") {
                    List {
                        ListItem {
                            Text(content: "Automatically builds projects before deployment")
                        }
                        ListItem {
                            Text(content: "Uses Wrangler CLI for Cloudflare Workers and Pages deployment")
                        }
                        ListItem {
                            Text(content: "Supports multiple environments via wrangler.toml configuration")
                        }
                    }
                }
            }
        })
    }
}

impl Default for MoonflareUI {
    fn default() -> Self {
        Self::new()
    }
}
