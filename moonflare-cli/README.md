# Moonflare CLI

A CLI utility for managing Cloudflare-focused monorepos with excellent developer experience, powered by Moon.

## Features

- **🚀 Quick Setup**: Initialize new monorepos with a single command
- **📦 Multi-Project Support**: Astro sites, React apps, Cloudflare Workers, Durable Objects, and Rust WASM libraries  
- **🔧 Built-in Tooling**: Integrated linting, formatting, and building via Biome and Moon
- **⚡ WASM Integration**: Seamless Rust → WASM → TypeScript workflow
- **☁️ Cloudflare Ready**: Optimized for Cloudflare Pages, Workers, and Durable Objects deployment

## Installation

```bash
# Build from source (for now)
git clone https://github.com/moonflare-dev/moonflare
cd moonflare/moonflare-cli
cargo build --release
```

## Quick Start

```bash
# Initialize a new monorepo
moonflare init my-project

# Add projects
cd my-project
moonflare add astro my-site          # Astro static site
moonflare add react my-app           # React application  
moonflare add durable-object my-do   # Durable Object
moonflare add crate my-math          # Rust WASM library

# Build everything
moonflare build

# Deploy to Cloudflare
moonflare deploy
```

## Commands

### `moonflare init <name>`
Initialize a new Cloudflare-focused monorepo with Moon configuration.

### `moonflare add <type> <name>`
Add a new project to the monorepo. Supported types:
- `astro` - Astro static site (deployed to Cloudflare Pages)
- `react` - React + Vite application (deployed to Cloudflare Pages) 
- `durable-object` - Cloudflare Durable Object worker
- `crate` - Rust library compiled to WebAssembly

### `moonflare build [project]`
Build all projects or a specific project using Moon's dependency graph.

### `moonflare dev [project]`  
Start development servers for all projects or a specific project.

### `moonflare deploy [project]`
Deploy projects to Cloudflare using Wrangler CLI.

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

Moonflare creates a Moon-managed monorepo optimized for Cloudflare deployment:

1. **Rust crates** compile to WebAssembly targeting `wasm32-unknown-unknown`
2. **Shared WASM** project collects all `.wasm` files for TypeScript consumption  
3. **TypeScript projects** (React/Astro) automatically depend on compiled WASM
4. **Moon** orchestrates the build pipeline with proper dependency ordering
5. **Wrangler** handles deployment to Cloudflare's edge network

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes  
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.