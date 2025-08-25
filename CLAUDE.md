# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Moonflare is a CLI utility for creating and managing Cloudflare-focused monorepos with excellent developer experience. The repository contains:

- **`moonflare-cli/`** - The main Rust CLI binary that generates and manages monorepos
- **`templates/`** - Legacy templates (kept for reference, now embedded in CLI)
- **Generated monorepos** have this structure:
  - **`crates/`** - Rust libraries that compile to WebAssembly (WASM32 target)
  - **`apps/`** - React frontend applications with Vite
  - **`sites/`** - Astro static sites
  - **`workers/`** - Cloudflare Workers with Durable Objects
  - **`shared-wasm/`** - Compiled WASM artifacts collected from crates

### CLI Architecture

The Moonflare CLI is built in Rust with:
- **Clap** for command parsing and help generation
- **Handlebars** for template processing with `convert_case` for string transformations
- **Embedded templates** for all project types (Astro, React, Durable Objects, Rust crates)
- **Moon integration** for task orchestration and dependency management
- **Cloudflare integration** via Wrangler CLI for deployment

## Common Commands

### CLI Development Commands
- `cargo build --release` - Build the Moonflare CLI binary
- `cargo test` - Run CLI tests
- `cargo clippy` - Lint CLI code
- `./moonflare-cli/target/release/moonflare --help` - Test CLI help

### CLI Usage Commands
- `moonflare init <name>` - Initialize new Cloudflare monorepo
- `moonflare add <type> <name>` - Add project (astro, react, durable-object, crate)
- `moonflare build [project]` - Build all projects or specific project
- `moonflare dev [project]` - Start development server(s)
- `moonflare deploy [project]` - Deploy to Cloudflare

### Generated Monorepo Commands (within created projects)
- `moon run :build` - Build all projects in generated monorepo
- `moon run :test` - Run all tests in generated monorepo  
- `moon run :lint` - Lint all projects in generated monorepo
- `moon run <project>:<task>` - Run specific project task

## Project Types and Tooling

### Rust Crates (`crates/`)
- Language: Rust
- Tasks: `format` (cargo fmt), `lint` (cargo clippy), `test` (cargo test), `build` (cargo build --release --target wasm32-unknown-unknown)
- Build output: WASM files in `target/wasm32-unknown-unknown/release/`

### TypeScript Projects (`apps/`, `sites/`, `workers/`)
- Language: TypeScript
- Package manager: pnpm
- Linting/Formatting: Biome
- Tasks: `build` (pnpm build), `lint` (pnpm biome lint), `format` (pnpm biome format), `check` (pnpm biome check)
- All depend on `shared-wasm:gather` to access compiled WASM

### Frontend Stack Differences
- **Astro sites**: Static site generation, layer: application, stack: frontend
- **React apps**: Vite + React, layer: application, stack: frontend  
- **Durable Object workers**: Cloudflare Workers, layer: application, stack: backend

## Advanced Moon Features

### Task Types and Modes
- **Build tasks**: Compile/build operations with outputs and caching
- **Run tasks**: Execute applications or scripts
- **Test tasks**: Run test suites
- **Local tasks**: Only run locally (not in CI), caching disabled
- **Internal tasks**: Used as dependencies, hidden from user
- **Persistent tasks**: Long-running processes (servers/watchers), run last in dependency graph

### Project Structure
- **Project layers**: library, application, tool (affects task inheritance)
- **Project stacks**: frontend, backend (provides context for tooling)
- **Project aliases**: Language-inferred secondary names (e.g., package.json name)

### Configuration Inheritance
- Tasks can be defined at workspace level (`.moon/tasks.yml`) and inherited by projects
- Project-level `moon.yml` can override or extend workspace tasks
- Supports command vs script distinction (commands are single binaries, scripts support pipes/shells)

### Performance Features
- **Smart hashing**: Deterministic builds based on inputs
- **Incremental builds**: Only rebuild changed projects
- **Remote caching**: Share build artifacts across environments
- **Parallel execution**: Tasks run concurrently based on dependency graph

## Development Workflow

### Working on the CLI
1. Make changes to Rust code in `moonflare-cli/src/`
2. Test with `cargo check` and `cargo clippy`
3. Build with `cargo build --release`
4. Test CLI functionality with `./moonflare-cli/target/release/moonflare`

### CLI Template Development
1. Templates are in `moonflare-cli/templates/*.template`
2. Use `FILE:` prefix for each file in template
3. Use `{{variable}}` syntax for Handlebars substitution
4. Templates are embedded at compile time via `include_str!`

### Generated Monorepo Workflow (for end users)
1. Initialize: `moonflare init my-project`
2. Add projects: `moonflare add react my-app` 
3. WASM crates automatically integrate with TypeScript projects
4. Projects without crates build normally (no WASM gathering required)
5. Projects with crates automatically trigger WASM compilation and gathering
6. Use Moon's task dependencies for proper build ordering
7. Deploy with `moonflare deploy` using Wrangler integration

### Key Integration Points
- **Moon CLI**: Automatically installed via proto if missing
- **Wrangler**: Required for Cloudflare deployment (manual install)
- **WASM Pipeline**: Rust → wasm32-unknown-unknown → shared-wasm/ → TypeScript imports
- **Modern Cloudflare Workers**: Uses `extends DurableObject` from "cloudflare:workers"
- **pnpm Workspaces**: Proper workspace configuration via pnpm-workspace.yaml