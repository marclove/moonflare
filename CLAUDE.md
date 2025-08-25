# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Moonflare is a Moon-managed monorepo for Cloudflare-focused web development with WASM integration. The repository is structured as:

- **`crates/`** - Rust libraries that compile to WebAssembly (WASM32 target)
- **`apps/`** - React frontend applications with Vite
- **`sites/`** - Astro static sites
- **`workers/`** - Cloudflare Workers with Durable Objects
- **`shared-wasm/`** - Compiled WASM artifacts collected from crates
- **`templates/`** - Moon project templates for scaffolding new projects

### WASM Build Pipeline

Rust crates build to `wasm32-unknown-unknown` target. The `shared-wasm` project gathers all `.wasm` files from `crates/*/target/wasm32-unknown-unknown/release/*.wasm` and makes them available to TypeScript projects as dependencies.

## Common Commands

All commands use Just as a task runner and Moon as the monorepo tool:

### Development Commands
- `just build` - Build all projects
- `just test` - Run all tests
- `just lint` - Lint all projects
- `just format` - Format all code
- `just check` - Run all checks (Biome for TS, Clippy for Rust)
- `just fix` - Auto-fix linting issues

### Project Creation
- `just astro <name>` - Create new Astro site in `sites/<name>`
- `just react <name>` - Create new React app in `apps/<name>`
- `just durable-object <name>` - Create new Durable Object worker in `workers/<name>`
- `just crate <name>` - Create new Rust crate in `crates/<name>`

### Moon Commands
Use `moon` directly for more granular control:
- `moon run <project>:<task>` - Run specific task
- `moon :lint` - Run lint across all projects
- `moon <project>:build` - Build specific project

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

1. When creating new Rust crates, they automatically get added to the `shared-wasm/moon.yml` dependency chain
2. WASM builds are automatically gathered into `shared-wasm/` directory
3. TypeScript projects automatically depend on the gathered WASM files
4. Use Moon's task dependencies to ensure proper build order
5. Leverage `moon run` for granular task execution with dependency resolution
6. Use project aliases interchangeably with project names in commands