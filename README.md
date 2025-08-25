# 🚀 Moonflare CLI

**A CLI utility for creating and managing Cloudflare-focused monorepos with excellent developer experience.**

Moonflare combines the power of [Moon](https://moonrepo.dev) for monorepo management with Cloudflare's edge platform, providing seamless integration between Rust WebAssembly, TypeScript applications, and Cloudflare services.

## ✨ Features

- **🏗️ Instant Setup**: Create production-ready monorepos with a single command
- **🔧 Multi-Stack Support**: Astro sites, React apps, Cloudflare Workers, Durable Objects, and Rust WASM libraries
- **⚡ WASM-First**: Seamless Rust → WebAssembly → TypeScript workflow with automatic dependency management
- **🎯 Cloudflare Optimized**: Built-in deployment to Workers, Pages, and Durable Objects using modern Workers Static Assets
- **📦 Smart Dependencies**: Automatic build ordering and task orchestration via Moon's dependency graph
- **🛠️ Integrated Tooling**: Biome formatting/linting, TypeScript, and Rust toolchains with beautiful Starbase UI
- **🚀 Zero Config**: Everything works out of the box with sensible defaults
- **🏷️ Intelligent Builds**: Projects build efficiently with or without WASM dependencies
- **🧪 Environment Support**: Deploy to different environments using `WRANGLER_ENV`

## 🚀 Installation

```bash
# Clone and build from source
git clone https://github.com/moonflare-dev/moonflare
cd moonflare
cargo build --release

# The binary will be at target/release/moonflare
# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

## ⚡ Quick Start

### Create Your First Project

```bash
# Initialize a new monorepo
moonflare init my-awesome-project
cd my-awesome-project

# Add different project types
moonflare add astro marketing-site       # Static marketing site
moonflare add react admin-dashboard      # Interactive SPA dashboard
moonflare add durable-object user-state  # Stateful edge service
moonflare add crate image-processing     # High-performance WASM library

# Build everything
moonflare build

# Start development servers
moonflare dev

# Deploy to Cloudflare
moonflare deploy
```

## 📚 Commands

Moonflare leverages [Moon's task orchestration](https://moonrepo.dev/) for intelligent dependency management and parallel execution. Each command can target all projects or specific ones.

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `moonflare init <name> [OPTIONS]` | Create new monorepo | `moonflare init my-project [--force]` |
| `moonflare add <type> <name>` | Add new project | `moonflare add react my-app` |
| `moonflare build [project]` | Build projects | `moonflare build [my-app]` |
| `moonflare dev [project]` | Start dev servers | `moonflare dev [my-app]` |
| `moonflare deploy [project] [OPTIONS]` | Deploy to Cloudflare | `moonflare deploy [--env staging]` |

### Project Types

| Type | Description | Directory | Deploys To |
|------|-------------|-----------|------------|
| `astro` | Static site with Astro | `sites/` | Cloudflare Workers (Static Assets) |
| `react` | React SPA with Vite + TypeScript | `apps/` | Cloudflare Workers (Static Assets) |
| `durable-object` | Stateful edge service with TypeScript | `workers/` | Cloudflare Workers |
| `crate` | Rust library → WASM | `crates/` | Embedded in TypeScript projects |

## 🏗️ Project Structure

Moonflare creates a Moon-managed monorepo with this structure:

```
my-project/
├── .moon/                 # Moon workspace configuration
│   ├── workspace.yml      # Moon workspace settings
│   └── toolchain.yml      # Toolchain versions (Node, Rust, etc.)
├── apps/                  # React applications
│   └── admin-dashboard/
├── sites/                 # Astro static sites
│   └── marketing-site/
├── workers/               # Cloudflare Workers & Durable Objects
│   └── user-state/
├── crates/                # Rust WASM libraries
│   └── image-processing/
├── shared-wasm/           # Compiled WASM artifacts (auto-generated)
│   ├── moon.yml          # WASM collection task
│   └── *.wasm           # Generated WASM files
├── package.json          # Root package.json with workspaces
├── justfile             # Task runner shortcuts
└── README.md
```

## 🔄 WASM Integration Workflow

Moonflare's killer feature is seamless WASM integration with intelligent conditional builds:

1. **Write Rust**: Create high-performance libraries in `crates/`
2. **Collect Artifacts**: `shared-wasm` project gathers all `.wasm` files (only when crates exist)
3. **Import in TypeScript**: WASM automatically available in React/Astro projects
4. **Deploy Together**: Everything deployed to Cloudflare's edge
5. **Graceful Builds**: Projects without WASM dependencies build normally without errors

```rust
// crates/math/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u32 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
```

```typescript
// apps/my-app/src/utils/math.ts
import init, { fibonacci } from '/shared-wasm/math.wasm';

await init();
const result = fibonacci(40); // Blazingly fast! 🔥
```

## 🏛️ Architecture

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

## 🌐 Cloudflare Integration

Moonflare is optimized for Cloudflare's modern developer platform:

- **Workers Static Assets**: Deploy Astro and React apps with zero config using the latest Workers approach
- **Modern Workers**: TypeScript-first with full type safety using `extends DurableObject` syntax
- **Durable Objects**: Stateful services at the edge with latest runtime type generation
- **High-Performance WASM**: Rust code running on V8 with conditional compilation
- **Service Integration**: Easy integration with KV, R2, D1, and other Cloudflare services
- **Smart Builds**: Only compiles WASM when crates are present, graceful fallbacks otherwise

## 🔧 Development

### Prerequisites

- **Rust** 1.75+ with `wasm32-unknown-unknown` target
- **Node.js** 18+ (managed by proto)
- **Moon CLI** (auto-installed via proto if missing)
- **pnpm**: Used for JavaScript/TypeScript project management
- **Wrangler**: Required for Cloudflare deployment (`npm install -g wrangler`)

### Building the CLI

```bash
# Build in release mode
cargo build --release

# Test the CLI
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

# CLI development tasks
just cli-dev               # Quick development check (check + lint + test)
just cli-release           # Full release preparation (format + lint + test + build)
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

## 🛠️ Technology Stack

- **Monorepo Management**: [Moon](https://moonrepo.dev)
- **Package Management**: pnpm with workspaces
- **Frontend**: Astro, React 19, Vite 6
- **Backend**: Cloudflare Workers, Durable Objects (modern `extends DurableObject` syntax)
- **WASM**: Rust with wasm-bindgen, conditional compilation
- **Linting/Formatting**: Biome
- **Deployment**: Wrangler CLI with Workers Static Assets
- **Task Running**: Just + Moon
- **String Processing**: convert_case for reliable template transformations

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch  
3. Make your changes
4. Run `cargo test` and `cargo clippy`
5. Test with real projects to ensure templates work correctly
6. Submit a pull request

## 📖 Examples

Check out the [examples](./examples) directory for complete project templates:

- **E-commerce Site**: Astro + React + Durable Objects + Rust WASM
- **SaaS Dashboard**: React + Workers + D1 Database  
- **Content Site**: Astro + KV + Image Optimization

## 📄 License

MIT License - see [LICENSE](./LICENSE) for details.

---

<div align="center">

**Built with ❤️ for the Cloudflare ecosystem**

[Examples](./examples) • [Contributing](./CONTRIBUTING.md) • [Issues](https://github.com/moonflare-dev/moonflare/issues)

</div>