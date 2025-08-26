# Local Testing Guide

This guide covers how to test Moonflare locally using Docker to replicate the CI environment before pushing to GitHub.

## Quick Start

```bash
# Quick local checks (fast, no Docker required)
just pre-check

# Comprehensive pre-release validation (uses Docker, mirrors CI)
just pre-release-check
```

## Available Testing Commands

### 🚀 Pre-Release Checks

| Command | Description | Time | Requirements |
|---------|-------------|------|--------------|
| `just pre-check` | Quick local validation | ~30s | Local Rust toolchain |
| `just pre-release-check` | Full CI-equivalent validation | ~5min | Docker |

### 🐳 Docker Testing Commands

| Command | Description | Use Case |
|---------|-------------|----------|
| `just docker-test-musl` | Test musl cross-compilation | Validate musl builds work |
| `just docker-test-cross` | Test ARM64 cross-compilation | Validate cross tool works |
| `just docker-test-logic` | Test build target logic | Validate routing logic |

### 🛠️ Docker Management

| Command | Description |
|---------|-------------|
| `just docker-build-full` | Build complete test environment |
| `just docker-clean` | Remove all test images |

## Testing Workflow

### Before Every PR/Push

```bash
# 1. Quick validation (always run this first)
just pre-check

# 2. Full validation if making release-related changes
just pre-release-check
```

### When Debugging CI Issues

```bash
# Test specific cross-compilation scenarios
just docker-test-musl      # Test x86_64-musl builds
just docker-test-cross     # Test ARM64 cross-compilation
just docker-test-logic     # Test build target routing
```

### Before Releases

```bash
# Comprehensive pre-release validation
just pre-release-check

# This validates:
# ✅ Code formatting and linting
# ✅ Full test suite passes
# ✅ Cross-compilation setup works
# ✅ Working directory is clean
# ✅ All build targets route correctly
```

## Understanding the Test Environment

### Docker Images

The test setup uses multi-stage Docker builds:

- **`moonflare-test:base`** - Basic Ubuntu + Rust + just
- **`moonflare-test:musl`** - Base + musl-tools for x86_64-musl
- **`moonflare-test:cross`** - Base + cross tool for ARM64
- **`moonflare-test:full`** - Complete environment (all targets)

### Target Testing Matrix

| Target | Test Command | Environment | Notes |
|--------|-------------|-------------|-------|
| `x86_64-unknown-linux-gnu` | Local cargo | Any platform | Native builds |
| `x86_64-unknown-linux-musl` | `docker-test-musl` | Linux+Docker | Uses musl-gcc |
| `aarch64-unknown-linux-gnu` | `docker-test-cross` | Linux+Docker | Uses cross tool |
| macOS/Windows targets | Local cargo | Native platform | Platform-specific |

## Troubleshooting

### Docker Issues

**"Cannot connect to Docker daemon"**
```bash
# Start Docker Desktop or ensure Docker daemon is running
docker --version
```

**"Image not found"**
```bash
# Rebuild the Docker image
just docker-build-full
```

**"Platform not supported"**
```bash
# Ensure you're on x86_64 or have emulation enabled
docker buildx ls
```

### Build Issues

**OpenSSL errors in musl builds**
- Expected during testing - musl builds require special OpenSSL setup
- CI handles this with proper dependencies

**Cross compilation fails**
- Ensure Docker has sufficient memory (4GB+ recommended)
- Check that cross tool installed correctly: `docker run --rm moonflare-test:cross cross --version`

### Performance Tips

**Speed up Docker builds:**
```bash
# Build base image once, reuse for multiple tests
just docker-build-base

# Clean up between test runs
just docker-clean
```

**Parallel testing:**
```bash
# Run quick checks while Docker builds in background
just pre-check & just docker-build-full && wait
```

## Integration with CI

The local Docker environment closely mirrors the GitHub Actions setup:

| CI Step | Local Equivalent | Command |
|---------|------------------|---------|
| Code quality | Format + lint checks | `cargo fmt --check && cargo clippy` |
| Test suite | Release test build | `cargo test --release` |
| Cross-compilation | Docker validation | `just docker-test-*` |
| Release prep | Working dir clean | `git status --porcelain` |

### Key Differences

- **CI** uses real cross-compilation, **local Docker** simulates it
- **CI** runs on Ubuntu 24.04, **local Docker** matches this exactly
- **CI** installs tools fresh each time, **Docker** caches for speed

## Best Practices

1. **Always run `just pre-check`** before pushing
2. **Run `just pre-release-check`** before creating releases
3. **Use specific Docker tests** when debugging cross-compilation
4. **Clean Docker images periodically** to save disk space
5. **Test on the actual target platforms** for final validation

## Advanced Usage

### Custom Docker Testing

```bash
# Run custom commands in test environment
docker run --rm -v $(pwd):/workspace moonflare-test:full bash -c "
    export RUNNER_OS=Linux
    cargo build --release --target x86_64-unknown-linux-musl
"

# Interactive debugging
docker run --rm -it -v $(pwd):/workspace moonflare-test:full bash
```

### CI Debugging

```bash
# Replicate exact CI environment
docker run --rm --platform linux/amd64 \
  -v $(pwd):/workspace \
  -e RUNNER_OS=Linux \
  moonflare-test:full bash -c "
    set -e
    cargo clippy -- -D warnings
    cargo test --release
    just build-target x86_64-unknown-linux-musl
  "
```

This testing setup ensures that issues are caught locally before they reach CI, significantly speeding up the development cycle.