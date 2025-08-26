# Contributing to Moonflare

Thank you for your interest in contributing to Moonflare! This guide will help you get started with developing and contributing to the project.

## Prerequisites

- **Rust** 1.75+ with `wasm32-unknown-unknown` target
- **Git** for version control
- **GitHub account** for contributing

### Optional (for testing generated projects)
- **Node.js** 18+ 
- **pnpm** package manager
- **Wrangler CLI** for Cloudflare deployment testing

## Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/moonflare-dev/moonflare.git
   cd moonflare
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run tests:**
   ```bash
   cargo test --release
   ```

4. **Check code formatting:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## Project Structure

```
moonflare/
├── src/                    # Main source code
│   ├── commands/          # CLI command implementations
│   ├── templates/         # Project templates (embedded)
│   ├── utils/            # Utility functions
│   └── ui.rs             # CLI user interface
├── tests/                 # Integration tests
├── .github/              # GitHub Actions workflows
└── scripts/              # Development scripts
```

## Development Workflow

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the project conventions

3. **Add tests** for new functionality

4. **Run the test suite:**
   ```bash
   cargo test --release
   ```

5. **Check code quality:**
   ```bash
   cargo fmt
   cargo clippy --fix
   ```

### Testing

We have several types of tests:

- **Unit tests**: Run with `cargo test --lib`
- **Integration tests**: Run with `cargo test --test <test_name>`
- **All tests**: Run with `cargo test --release`

**Important**: Some tests require Node.js dependencies to be installed. If you get build failures related to `vite` or other Node tools, this is expected in a clean environment.

### Code Style

- **Formatting**: Use `cargo fmt` to format code
- **Linting**: Address all `cargo clippy` warnings
- **Comments**: Add comments for complex logic, but prefer self-documenting code
- **Error Handling**: Use `anyhow::Result` for commands, `miette` for user-facing errors
- **UI**: Use Starbase components for consistent CLI output

### Adding New Commands

1. **Create the command module** in `src/commands/`
2. **Add the module** to `src/commands/mod.rs`
3. **Update the CLI** in `src/main.rs`:
   - Add to `Commands` enum
   - Add help handling
   - Add command execution
4. **Add help documentation** in `src/ui.rs`
5. **Create integration tests** in `tests/`

Example command structure:
```rust
pub struct MyCommand {
    ui: MoonflareUI,
}

impl MyCommand {
    pub fn new() -> Self {
        Self {
            ui: MoonflareUI::new(),
        }
    }

    pub async fn execute(&self, /* args */) -> anyhow::Result<()> {
        // Implementation
        Ok(())
    }
}
```

### Adding New Project Types

1. **Create template file** in `src/templates/`
2. **Update template engine** in `src/templates/embedded.rs`
3. **Add project type** to `ProjectType` enum in tests
4. **Add comprehensive tests** covering the new project type

## Submitting Changes

1. **Push your branch** to your fork
2. **Create a Pull Request** with:
   - Clear description of changes
   - Reference to related issues
   - Screenshots for UI changes
   - Test results

3. **Address review feedback** promptly

## Release Process

Releases are handled automatically via GitHub Actions:

1. **Create a release** using the release script:
   ```bash
   ./scripts/release.sh
   ```

2. **GitHub Actions** will:
   - Run tests
   - Build for all platforms
   - Create GitHub release
   - Generate release notes
   - Update Homebrew (for stable releases)

## Debugging Tips

### CLI Testing
```bash
# Build and test locally
cargo build --release
./target/release/moonflare --help

# Test in temporary directory
cd /tmp
./path/to/moonflare init test-project
cd test-project
./path/to/moonflare add react frontend
```

### Template Testing
Templates are embedded at compile time. After modifying templates:
```bash
cargo build --release  # Recompile to embed changes
```

### Integration Test Debugging
```bash
# Run specific test with output
cargo test --test rename_command_tests -- --nocapture

# Run tests without timeout issues
cargo test --release
```

## Common Issues

### Test Failures
- **Node.js missing**: Property tests may fail without Node.js/pnpm installed
- **Network timeouts**: Some integration tests have generous timeouts
- **Wrangler missing**: Deployment tests expect Wrangler CLI

### Build Issues
- **Starbase dependencies**: Ensure all UI dependencies are properly linked
- **Template compilation**: Templates are compiled into the binary

## Getting Help

- **Documentation**: Check the README and in-code comments
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our community Discord (link in README)

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive environment for all contributors.

Thank you for contributing to Moonflare! 🚀