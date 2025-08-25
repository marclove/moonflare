# 🚀 Moonflare

**A CLI utility for creating and managing Cloudflare-focused monorepos with excellent developer experience.**

Moonflare combines the power of [Moon](https://moonrepo.dev) for monorepo management with Cloudflare's edge platform, providing seamless integration between Rust WebAssembly, TypeScript applications, and Cloudflare services.

## ✨ Features

- **🏗️ Instant Setup**: Create production-ready monorepos in seconds
- **🔧 Multi-Stack Support**: Astro sites, React apps, Cloudflare Workers, Durable Objects, and Rust WASM libraries
- **⚡ WASM-First**: Seamless Rust → WebAssembly → TypeScript workflow
- **🎯 Cloudflare Optimized**: Built-in deployment to Pages, Workers, and Durable Objects
- **📦 Smart Dependencies**: Automatic dependency management via Moon's task system
- **🛠️ Integrated Tooling**: Biome formatting/linting, TypeScript, and Rust toolchains
- **🚀 Zero Config**: Everything works out of the box with sensible defaults
- **🏷️ Intelligent Builds**: Projects build efficiently with or without WASM dependencies

## 🚀 Quick Start

### Installation

```bash
# Clone and build from source
git clone https://github.com/moonflare-dev/moonflare.git
cd moonflare/moonflare-cli
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

### Create Your First Project

```bash
# Initialize a new monorepo
moonflare init my-awesome-project
cd my-awesome-project

# Add different project types
moonflare add astro marketing-site       # Static marketing site
moonflare add react admin-dashboard      # Interactive dashboard
moonflare add durable-object user-state  # Stateful edge service
moonflare add crate image-processing     # High-performance WASM library

# Build everything
moonflare build

# Start development
moonflare dev

# Deploy to Cloudflare
moonflare deploy
```

## 📚 Commands

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `moonflare init <name>` | Create new monorepo | `moonflare init my-project` |
| `moonflare add <type> <name>` | Add new project | `moonflare add react my-app` |
| `moonflare build [project]` | Build projects | `moonflare build` |
| `moonflare dev [project]` | Start dev servers | `moonflare dev my-app` |
| `moonflare deploy [project]` | Deploy to Cloudflare | `moonflare deploy` |

### Project Types

| Type | Description | Deploys To |
|------|-------------|------------|
| `astro` | Static site with Astro | Cloudflare Pages |
| `react` | React app with Vite | Cloudflare Pages |
| `durable-object` | Stateful edge service | Cloudflare Workers |
| `crate` | Rust library → WASM | Embedded in TypeScript projects |

## 🏗️ Project Structure

Moonflare creates a Moon-managed monorepo with this structure:

```
my-project/
├── .moon/
│   ├── workspace.yml      # Moon workspace configuration
│   └── toolchain.yml      # Toolchain versions (Node, Rust, etc.)
├── apps/                  # React applications
│   └── admin-dashboard/
├── sites/                 # Astro static sites
│   └── marketing-site/
├── workers/               # Cloudflare Workers & Durable Objects
│   └── user-state/
├── crates/                # Rust WASM libraries
│   └── image-processing/
├── shared-wasm/           # Compiled WASM artifacts
│   ├── moon.yml          # WASM collection task
│   └── *.wasm           # Generated WASM files
├── package.json          # Root package.json with workspaces
├── justfile             # Task shortcuts
└── README.md
```

## 🔄 WASM Integration Workflow

Moonflare's killer feature is seamless WASM integration with intelligent conditional builds:

1. **Write Rust**: Create high-performance libraries in `crates/`
2. **Collect Artifacts**: `shared-wasm` project gathers all `.wasm` files (only when crates exist)
3. **Import in TypeScript**: WASM automatically available in React/Astro projects
4. **Deploy**: Everything deployed together to Cloudflare's edge
5. **Graceful Builds**: Projects without WASM dependencies build normally without errors

```rust
// crates/math/src/lib.rs
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

## 🛠️ Technology Stack

- **Monorepo Management**: [Moon](https://moonrepo.dev)
- **Package Management**: pnpm with workspaces
- **Frontend**: Astro, React 19, Vite 6
- **Backend**: Cloudflare Workers, Durable Objects (modern `extends DurableObject` syntax)
- **WASM**: Rust with wasm-bindgen, conditional compilation
- **Linting/Formatting**: Biome
- **Deployment**: Wrangler CLI
- **Task Running**: Just + Moon
- **String Processing**: convert_case for reliable template transformations

## 🌐 Cloudflare Integration

Moonflare is optimized for Cloudflare's developer platform:

- **Pages**: Deploy Astro and React apps with zero config
- **Workers**: TypeScript-first with full type safety using modern patterns
- **Durable Objects**: Stateful services at the edge with latest `extends DurableObject` syntax
- **WASM**: High-performance Rust code running on V8 with conditional compilation
- **KV/R2/D1**: Easy integration with Cloudflare services
- **Smart Builds**: Only compiles WASM when crates are present, graceful fallbacks otherwise

## 🔧 Development

### Prerequisites

- **Rust** 1.75+ with `wasm32-unknown-unknown` target
- **Node.js** 18+ (managed by proto)
- **Moon CLI** (auto-installed via proto)

### Building from Source

```bash
git clone https://github.com/moonflare-dev/moonflare.git
cd moonflare/moonflare-cli
cargo build --release
./target/release/moonflare --help
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test && cargo clippy`
5. Submit a pull request

## 📖 Examples

Check out the [examples](./examples) directory for complete project templates:

- **E-commerce Site**: Astro + React + Durable Objects + Rust WASM
- **SaaS Dashboard**: React + Workers + D1 Database
- **Content Site**: Astro + KV + Image Optimization

## 🤝 Community

- **Discord**: [Join our community](https://discord.gg/moonflare)
- **GitHub Discussions**: [Ask questions](https://github.com/moonflare-dev/moonflare/discussions)
- **Twitter**: [@MoonflareHQ](https://twitter.com/MoonflareHQ)

## 📄 License

MIT License - see [LICENSE](./LICENSE) for details.

---

<div align="center">

**Built with ❤️ for the Cloudflare ecosystem**

[Documentation](https://moonflare.dev) • [Examples](./examples) • [Contributing](./CONTRIBUTING.md)

</div>
