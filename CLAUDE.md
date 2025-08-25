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

The Moonflare CLI is built in Rust with modern error handling and beautiful UI:
- **Clap** for command parsing and subcommand structure
- **Miette** for enhanced error reporting with context and suggestions
- **Starbase** for beautiful CLI output with consistent formatting and colors
- **Handlebars** for template processing with `convert_case` for string transformations
- **Embedded templates** for all project types (Astro, React, Durable Objects, Rust crates)
- **Moon integration** for task orchestration and dependency management
- **Cloudflare integration** via Wrangler CLI for deployment with environment support
- **Custom help system** that bypasses Clap to provide rich, formatted help output

## Common Commands

### CLI Development Commands
- `cargo build --release` - Build the Moonflare CLI binary
- `cargo test` - Run CLI tests
- `cargo clippy` - Lint CLI code
- `./moonflare-cli/target/release/moonflare --help` - Test CLI help

### CLI Usage Commands
- `moonflare init <name> [--path <path>] [--force]` - Initialize new Cloudflare monorepo
- `moonflare add <type> <name>` - Add project (astro, react, durable-object, crate)
- `moonflare build [project]` - Build all projects or specific project
- `moonflare dev [project]` - Start development server(s)  
- `moonflare deploy [project] [--env <environment>]` - Deploy to Cloudflare
- `moonflare help [command]` - Beautiful help output via Starbase UI

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
- Package manager: pnpm with project-level install tasks
- Linting/Formatting: Biome
- Tasks: `install` (pnpm install), `build` (pnpm build), `lint` (pnpm biome lint), `format` (pnpm biome format), `check` (pnpm biome check), `deploy` (pnpm deploy)
- All projects with WASM dependencies automatically depend on `shared-wasm:gather`
- Deploy tasks depend on build tasks (`deps: ['~:build']`)

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
1. Initialize: `moonflare init my-project [--force]`
2. Add projects: `moonflare add react my-app` 
3. Projects automatically get proper Moon task configuration
4. WASM crates automatically integrate with TypeScript projects via dependencies
5. Projects without crates build normally (no WASM gathering required)
6. Projects with crates automatically trigger WASM compilation and gathering
7. Use Moon's task dependencies for proper build ordering and parallel execution
8. Deploy with `moonflare deploy [--env staging]` using Moon tasks → Wrangler

### Key Integration Points
- **Moon CLI**: Automatically installed via proto if missing
- **Wrangler**: Required for Cloudflare deployment (manual install)
- **WASM Pipeline**: Rust → wasm32-unknown-unknown → shared-wasm/ → TypeScript imports
- **Modern Cloudflare Workers**: Uses `extends DurableObject` from "cloudflare:workers"
- **pnpm Workspaces**: Proper workspace configuration via pnpm-workspace.yaml
- **Environment Variables**: Deployed via Moon's environment inheritance (WRANGLER_ENV)
- **Multiple Wrangler Configs**: Supports wrangler.toml, wrangler.json, and wrangler.jsonc

## UI and Error Handling System

### Starbase UI Components (`src/ui.rs`)
The CLI uses Starbase for consistent, beautiful output:
- **MoonflareUI struct**: Central UI manager with Console and theming
- **Error rendering**: `render_error()` with title, message, and suggestions
- **Success rendering**: `render_success()` for completion messages  
- **Section rendering**: `render_section()` for organized content
- **Help system**: Complete custom help bypassing Clap for rich formatting
- **Components used**: Container, Notice, Section, List, Entry, StyledText

### Error Handling Strategy
- **Miette integration**: Enhanced error reporting with context and suggestions
- **Custom error types**: `MoonflareError` with specific error variants
- **Helpful suggestions**: Non-empty directory errors suggest `--force` flag
- **Graceful fallbacks**: UI render errors fall back to process::exit(1)
- **Context preservation**: Maintain full error chains for debugging

## Current Dependencies (`Cargo.toml`)

### Core CLI Dependencies
- **clap**: Command-line argument parsing with derive macros
- **miette**: Enhanced error reporting with fancy formatting
- **tokio**: Async runtime for CLI operations
- **anyhow**: Error handling compatibility layer

### UI and Formatting
- **starbase**: Core framework for beautiful CLI output
- **starbase_console**: Console components with UI features
- **iocraft**: Low-level UI component system
- **colored**: Terminal color support

### Template Processing
- **handlebars**: Template engine for project generation
- **convert_case**: String case transformations (snake_case, PascalCase, etc.)
- **serde/serde_json/serde_yaml/toml**: Configuration file parsing

### File System and Networking
- **walkdir**: Directory traversal for template processing
- **fs_extra**: Extended file system operations
- **tempfile**: Temporary file handling for tests
- **reqwest**: HTTP client for external API calls
- **which**: Binary location detection

## Testing Strategy

### Test Structure
- **Integration tests**: Located in `tests/` directory (e.g., `init_dot_tests.rs`)
- **Property-based testing**: Uses `proptest` for generating test inputs
- **Temporary directories**: Tests use `tempfile` for isolated file system operations
- **Case-insensitive assertions**: Error message tests handle UI casing variations

### Key Test Patterns
```rust
// Case-insensitive error checking
let stderr_lower = stderr.to_lowercase();
assert!(stderr_lower.contains("directory is not empty"));

// Temporary directory setup
let temp_dir = tempfile::tempdir()?;
let target = temp_dir.path().join("test-project");
```

### Running Tests
- `cargo test` - Run all tests
- `cargo test init_` - Run init-specific tests  
- `cargo clippy` - Lint checks
- `cargo fmt` - Code formatting