# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Moonflare is a project template repository that provides Moon build system configurations for various technology stacks including Astro, React, Rust crates, and Cloudflare Durable Objects. The repository uses the Moon build system for task orchestration and Just for project setup automation.

## Key Commands

### Setup Commands
- `just` - Install dependencies (brew, proto, moon setup) and create project directories
- `moon setup` - Initialize the Moon build system

### Project Creation Commands
- `just create-astro-app <dest>` - Create a new Astro app in `sites/<dest>`
- `just create-react-app <dest>` - Create a new React app in `apps/<dest>`
- `just create-durable-object <dest>` - Create a new Durable Object worker in `workers/<dest>`
- `just create-crate <dest>` - Create a new Rust crate in `crates/<dest>`

### Common Moon Tasks
For TypeScript projects (Astro/React/Durable Objects):
- `moon run <project>:build` - Build the project using pnpm

For Rust projects:
- `moon run <project>:build` - Build WASM target with `cargo build --release --target wasm32-unknown-unknown`
- `moon run <project>:test` - Run tests with `cargo test`
- `moon run <project>:lint` - Run clippy linter
- `moon run <project>:format` - Format code with cargo fmt
- `moon run <project>:typecheck` - Type check with `cargo check`

## Architecture

### Project Structure
```
templates/
├── astro-moon/          # Template for Astro applications
├── react-moon/          # Template for React applications
├── durable-object-moon/ # Template for Cloudflare Durable Objects
└── crate-moon/          # Template for Rust crates
```

### Template System
Each template contains:
- `moon.yml` - Moon project configuration defining tasks, inputs, outputs, and dependencies
- `template.yml` - Moon template metadata

### Technology Stack Layers
- **Frontend Applications**: Astro and React projects in the `application` layer with `frontend` stack
- **Backend Applications**: Durable Objects in the `application` layer with `backend` stack
- **Libraries**: Rust crates in the `library` layer

### Build Dependencies
- TypeScript projects depend on `shared-wasm:gather` task
- WASM files are expected in `/shared-wasm/*.wasm`
- All TypeScript projects use pnpm as the package manager

### Prerequisites
The setup requires:
- proto (for Node.js and Rust version management)
- Moon build system
- brew (for dependency installation)
- cargo (for Rust projects)
- pnpm (for TypeScript projects)
