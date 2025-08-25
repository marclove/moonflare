# Moonflare CLI

A CLI utility for managing Cloudflare-focused monorepos with excellent developer experience, powered by [Moon](https://moonrepo.dev/).

## Features

- **Quick Setup**: Initialize new monorepos with a single command
- **Multi-Project Support**: Astro sites, React apps, Cloudflare Workers, Durable Objects, and Rust WASM libraries  
- **Beautiful CLI Interface**: Starbase-powered interface with rich formatting and helpful guidance
- **Built-in Tooling**: Integrated linting, formatting, and building via Biome and Moon
- **WASM Integration**: Seamless Rust → WASM → TypeScript workflow with automatic dependency management
- **Cloudflare Ready**: Optimized for Cloudflare Pages, Workers, and Durable Objects deployment
- **Environment Support**: Deploy to different environments using `WRANGLER_ENV`
- **Smart Dependencies**: Automatic build ordering and task orchestration via Moon

## Installation

```bash
# Build from source
git clone https://github.com/moonflare-dev/moonflare
cd moonflare/moonflare-cli
cargo build --release

# The binary will be at moonflare-cli/target/release/moonflare
```

## Prerequisites

- **Rust**: Required for building Moonflare CLI
- **Moon**: Automatically installed via [proto](https://moonrepo.dev/docs/proto) if not found
- **pnpm**: Used for JavaScript/TypeScript project management
- **Wrangler**: Required for Cloudflare deployment (`npm install -g wrangler`)

## Quick Start

```bash
# Initialize a new monorepo
moonflare init my-project

# Navigate to your project
cd my-project

# Add different types of projects
moonflare add astro marketing        # Astro static site
moonflare add react dashboard        # React application  
moonflare add durable-object storage # Durable Object
moonflare add crate math-utils       # Rust WASM library

# Build all projects (or specify one)
moonflare build                      # Build everything
moonflare build dashboard           # Build specific project

# Start development servers
moonflare dev                       # All projects  
moonflare dev marketing             # Specific project

# Deploy to Cloudflare
moonflare deploy                    # All deployable projects
moonflare deploy marketing          # Specific project
moonflare deploy --env staging marketing  # With environment
```

## Commands

Moonflare leverages [Moon's task orchestration](https://moonrepo.dev/) for intelligent dependency management and parallel execution. Each command can target all projects or specific ones.

### `moonflare init <name> [OPTIONS]`
Initialize a new Cloudflare-focused monorepo with Moon configuration and pnpm workspaces.

**Options:**
- `--path <PATH>` - Directory to create the monorepo in (default: current directory)  
- `--force` - Force initialization in non-empty directories

### `moonflare add <type> <name>`
Add a new project to the monorepo with embedded templates. Supported types:
- `astro` - Astro static site with Cloudflare Pages deployment
- `react` - React + Vite application with TypeScript and Biome
- `durable-object` - Cloudflare Durable Object worker with TypeScript
- `crate` - Rust library compiled to WebAssembly (wasm32-unknown-unknown target)

Each project includes Moon task configuration, Biome formatting/linting, and automatic WASM integration.

### `moonflare build [project]`
Build all projects or a specific project using Moon's dependency graph. WASM crates are automatically built before TypeScript projects that depend on them.

### `moonflare dev [project]`  
Start development servers for all projects or a specific project. Runs `astro dev`, `vite dev`, or `wrangler dev` as appropriate.

### `moonflare deploy [project] [OPTIONS]`
Deploy projects to Cloudflare using Wrangler. Only deploys projects with `wrangler.toml`, `wrangler.json`, or `wrangler.jsonc` configuration.

**Options:**
- `--env <ENV>` - Deploy to specific environment (sets `WRANGLER_ENV`)

## Project Structure

```
my-project/
├── .moon/                 # Moon workspace configuration
├── apps/                  # React applications
├── sites/                 # Astro static sites
├── workers/               # Cloudflare Workers & Durable Objects  
├── crates/                # Rust WASM libraries
├── shared-wasm/           # Compiled WASM artifacts
├── package.json           # Root package.json with workspaces
├── justfile              # Task runner shortcuts
└── README.md
```

## Architecture

Moonflare creates a Moon-managed monorepo optimized for Cloudflare deployment with intelligent dependency management:

### Core Technologies
- **CLI Framework**: Rust with Clap for argument parsing and Miette for enhanced error reporting
- **UI Framework**: Starbase for beautiful, consistent CLI output with rich formatting
- **Task Orchestration**: Moon for dependency graphs, incremental builds, and parallel execution
- **Package Management**: pnpm workspaces for JavaScript/TypeScript dependencies
- **Template Engine**: Handlebars with convert_case for project generation

### Build Pipeline
1. **Rust crates** compile to WebAssembly targeting `wasm32-unknown-unknown`
2. **Shared WASM** project (`shared-wasm:gather`) collects all `.wasm` files automatically
3. **TypeScript projects** (React/Astro/Durable Objects) depend on WASM via Moon tasks
4. **Moon** orchestrates the build pipeline with proper dependency ordering and caching
5. **Project-level dependencies** ensure devDependencies like Wrangler are available
6. **Wrangler** handles deployment with environment variable support

### Template System
Templates are embedded in the CLI binary using `include_str!` and processed with Handlebars:
- Variable substitution: `{{name}}`, `{{upper name}}`, `{{title name}}`
- Conditional blocks: `{{#if has_wasm}}...{{/if}}`  
- Multi-file templates: `FILE:` prefix for each file in the template

## Development

### Building the CLI
```bash
cd moonflare-cli
cargo build --release
./target/release/moonflare --help
```

### Running Tests
```bash
cargo test                    # Run all unit and integration tests
cargo clippy                  # Lint code
cargo fmt                     # Format code

# Smoke tests (deploy to real Cloudflare infrastructure)
cargo test --test deployment_smoke_tests -- --ignored
```

#### Deployment Smoke Tests
The repository includes comprehensive smoke tests that verify the complete deployment workflow by creating, building, and deploying projects to real Cloudflare infrastructure.

**Prerequisites for smoke tests:**
- Valid Cloudflare account with Workers/Pages enabled
- Wrangler CLI installed and authenticated (`wrangler login`)
- Network connectivity

**Available smoke tests:**
```bash
# Full workflow: create workspace, add all project types, build, deploy, verify  
cargo test --test deployment_smoke_tests test_full_deployment_workflow -- --ignored

# Individual project type deployments
cargo test --test deployment_smoke_tests test_individual_project_deployments -- --ignored

# Dependency resolution testing
cargo test --test deployment_smoke_tests test_dependency_resolution -- --ignored

# Run all smoke tests
cargo test --test deployment_smoke_tests -- --ignored
```

**Using Just tasks (from repo root):**
```bash
# All smoke tests (with safety warning)
just smoke-test

# Individual smoke tests
just smoke-test-full        # Full workflow test
just smoke-test-individual  # Individual project tests  
just smoke-test-deps        # Dependency resolution test
```

⚠️ **Warning**: Smoke tests deploy to real Cloudflare infrastructure and may incur usage costs. Use with caution in CI environments.

### Testing Generated Projects
```bash
# Test the full workflow
./target/release/moonflare init test-project
cd test-project
../target/release/moonflare add crate math
../target/release/moonflare add react app  
../target/release/moonflare build
```

## Contributing

1. Fork the repository
2. Create a feature branch  
3. Make your changes
4. Run `cargo test` and `cargo clippy`
5. Test with real projects to ensure templates work correctly
6. Submit a pull request

## Dependencies

- **Runtime**: Moon (auto-installed), pnpm, Wrangler (manual install)
- **Development**: Rust toolchain with `wasm32-unknown-unknown` target
- **CLI**: Clap, Miette, Starbase, Handlebars, Tokio

## License

MIT License - see LICENSE file for details.