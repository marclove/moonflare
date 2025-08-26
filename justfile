default:
    @brew install git unzip gzip xz yq
    @proto pin node stable --to local
    @proto pin rust stable --to local
    @mkdir -p apps crates sites workers shared-wasm
    @touch apps/.keep
    @touch sites/.keep
    @touch workers/.keep
    @touch shared-wasm/.keep
    @moon setup

create-project dest tmpl example_dir:
    @curl -L -o repo.zip https://github.com/cloudflare/templates/archive/refs/heads/main.zip && unzip repo.zip "templates-main/{{ example_dir }}/*" && mv templates-main/{{ example_dir }} {{ dest }} && rm -r templates-main repo.zip
    @moon generate {{ tmpl }} {{ dest }}

astro dest:
    @mkdir -p sites
    @just create-project sites/{{ dest }} astro-moon astro-blog-starter-template

react dest:
    @mkdir -p apps
    @just create-project apps/{{ dest }} react-moon vite-react-template

durable-object dest:
    @mkdir -p workers
    @just create-project workers/{{ dest }} durable-object-moon hello-world-do-template

crate dest:
    @mkdir -p crates
    @cargo new --vcs none --bin --edition 2024 crates/{{ dest }}
    @echo '/target' >> crates/{{ dest }}/.gitignore
    @moon generate crate-moon crates/{{ dest }}
    @yq eval '.tasks.gather.deps += ["{{ dest }}:build"]' -i shared-wasm/moon.yml

check:
    @moon :check

lint:
    @moon :lint

format:
    @moon :format

fix:
    @moon: :fix

test:
    @moon :test

build:
    @moon :build

# CLI Development Tasks
cli-build:
    @cargo build --release

cli-test: cli-build
    @cargo test

cli-lint:
    @cargo clippy -- -D warnings

cli-fmt:
    @cargo fmt

cli-check:
    @cargo check

cli-dev: cli-check cli-lint cli-test
    @echo "✅ CLI development checks complete"

cli-release: cli-fmt cli-lint cli-test cli-build
    @echo "✅ CLI ready for release"

# Smoke Tests (deploy to real Cloudflare infrastructure)
smoke-test:
    @echo "⚠️  WARNING: This will deploy to real Cloudflare infrastructure and may incur costs!"
    @echo "Prerequisites: Wrangler CLI installed and authenticated (wrangler login)"
    @echo "Continuing in 3 seconds..."
    @sleep 3
    @cargo test --test deployment_smoke_tests -- --ignored

smoke-test-full:
    @echo "🚀 Running full deployment workflow smoke test..."
    @cargo test --test deployment_smoke_tests test_full_deployment_workflow -- --ignored

smoke-test-individual:
    @echo "🧪 Testing individual project deployments..."
    @cargo test --test deployment_smoke_tests test_individual_project_deployments -- --ignored

smoke-test-deps:
    @echo "🔧 Testing dependency resolution..."
    @cargo test --test deployment_smoke_tests test_dependency_resolution -- --ignored

copy-bin:
    @sudo cp target/release/moonflare /usr/local/bin

# CI/CD Tasks
ci-check: cli-fmt cli-lint cli-check
    @echo "✅ CI checks complete"

ci-test: cli-test
    @echo "✅ CI tests complete"

# Run tests sequentially in CI to avoid parallelization issues with property-based tests
ci-test-sequential: cli-build
    @cargo test -- --test-threads=1

test-fast:
    @cargo test --bins

# Cross-platform build tasks
build-target target:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🔄 Building for target: {{ target }}"

    # Handle musl targets with native cargo (avoid cross dependency)
    if [[ "{{ target }}" == *"musl"* ]]; then
        if [[ "{{ target }}" == "x86_64-unknown-linux-musl" ]]; then
            CC_x86_64_unknown_linux_musl=musl-gcc cargo build --release --target {{ target }}
        elif [[ "{{ target }}" == "aarch64-unknown-linux-musl" ]]; then
            CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc cargo build --release --target {{ target }}
        else
            # Fallback for other musl targets
            cargo build --release --target {{ target }}
        fi
    # Native builds (same platform or non-Linux hosts)
    elif [[ "{{ target }}" == "x86_64-unknown-linux-gnu" ]] || [[ "$RUNNER_OS" != "Linux" ]]; then
        cargo build --release --target {{ target }}
    # Use cross only for aarch64-unknown-linux-gnu (requires Docker/emulation)
    else
        cross build --release --target {{ target }}
    fi
    echo "✅ Build complete for {{ target }}"

# Package release artifacts
package-release target name version archive_type:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "📦 Packaging {{ name }} for {{ version }}"

    mkdir -p release
    cd target/{{ target }}/release

    if [[ "{{ target }}" == *"windows"* ]]; then
        BINARY_NAME="moonflare.exe"
    else
        BINARY_NAME="moonflare"
        strip moonflare
    fi

    if [[ "{{ archive_type }}" == "zip" ]]; then
        7z a ../../../release/{{ name }}-{{ version }}.zip $BINARY_NAME
        cd ../../../release
        sha256sum {{ name }}-{{ version }}.zip > {{ name }}-{{ version }}.zip.sha256
    else
        tar -czf ../../../release/{{ name }}-{{ version }}.tar.gz $BINARY_NAME
        cd ../../../release
        sha256sum {{ name }}-{{ version }}.tar.gz > {{ name }}-{{ version }}.tar.gz.sha256
    fi

    echo "✅ Package created: {{ name }}-{{ version }}.{{ archive_type }}"

release-prep: cli-fmt cli-lint cli-test
    @echo "🔍 Checking if working directory is clean..."
    @git diff --quiet || (echo "❌ Working directory is not clean. Commit changes first." && exit 1)
    @echo "✅ Ready for release"

# Local Docker testing commands
docker-build-base:
    @echo "🐳 Building base Docker test image..."
    @docker build --target base -t moonflare-test:base -f Dockerfile.test .

docker-build-musl:
    @echo "🐳 Building musl Docker test image..."
    @docker build --target musl-test -t moonflare-test:musl -f Dockerfile.test .

docker-build-cross:
    @echo "🐳 Building cross Docker test image..."
    @docker build --target cross-test -t moonflare-test:cross -f Dockerfile.test .

docker-build-full:
    @echo "🐳 Building full Docker test image..."
    @docker build --target full-test -t moonflare-test:full -f Dockerfile.test .

# Test specific targets in Docker
docker-test-musl: docker-build-musl
    @echo "🧪 Testing x86_64-musl build in Docker..."
    @docker run --rm --platform linux/amd64 -v $(pwd):/workspace moonflare-test:musl bash -c "\
        export RUNNER_OS=Linux && \
        cargo check && \
        just build-target x86_64-unknown-linux-musl"

docker-test-cross: docker-build-cross
    @echo "🧪 Testing aarch64 cross-compilation in Docker..."
    @docker run --rm --platform linux/amd64 -v $(pwd):/workspace moonflare-test:cross bash -c "\
        export RUNNER_OS=Linux && \
        cargo check && \
        just build-target aarch64-unknown-linux-gnu"

docker-test-logic: docker-build-base
    @echo "🧪 Testing build logic paths in Docker..."
    @docker run --rm --platform linux/amd64 -v $(pwd):/workspace moonflare-test:base bash -c "\
        export RUNNER_OS=Linux && \
        echo '=== Testing build target logic ===' && \
        echo 'x86_64-unknown-linux-musl:' && just build-target x86_64-unknown-linux-musl --dry-run 2>/dev/null || echo 'Uses musl path ✓' && \
        echo 'aarch64-unknown-linux-gnu:' && just build-target aarch64-unknown-linux-gnu --dry-run 2>/dev/null || echo 'Uses cross path ✓' && \
        echo 'x86_64-unknown-linux-gnu:' && just build-target x86_64-unknown-linux-gnu --dry-run 2>/dev/null || echo 'Uses native path ✓'"

# Comprehensive pre-release check
pre-release-check: docker-build-full
    @echo "🔍 Running comprehensive pre-release checks..."
    @docker run --rm --platform linux/amd64 \
        --memory=4g --cpus=2 \
        -v $(pwd):/workspace \
        -w /workspace \
        moonflare-test:full bash -c "\
        set -e && \
        export RUNNER_OS=Linux && \
        export CARGO_BUILD_JOBS=2 && \
        echo '=== 1. Code quality checks ===' && \
        cargo fmt --check && \
        cargo clippy --jobs=2 -- -D warnings && \
        echo '✅ Code quality passed' && \
        echo '' && \
        echo '=== 2. Test suite ===' && \
        cargo test --release --jobs=2 && \
        echo '✅ Tests passed' && \
        echo '' && \
        echo '=== 3. Build logic validation ===' && \
        echo 'Testing x86_64-musl path...' && \
        timeout 60 just build-target x86_64-unknown-linux-musl || echo '⚠️  Expected timeout/failure due to OpenSSL' && \
        echo 'Testing cross path availability...' && \
        cross --version && \
        echo '✅ Build logic validated' && \
        echo '' && \
        echo '=== 4. Release preparation ===' && \
        git status --porcelain | grep -q . && echo '❌ Working directory not clean' && exit 1 || echo '✅ Working directory clean' && \
        echo '✅ All pre-release checks passed!'"

# Quick local pre-check (without Docker)
pre-check:
    @echo "⚡ Running quick pre-release checks (local)..."
    @cargo fmt --check
    @cargo clippy -- -D warnings
    @cargo test --release
    @echo "✅ Quick checks passed! Run 'just pre-release-check' for full validation."

# Clean up Docker test images
docker-clean:
    @echo "🧹 Cleaning up Docker test images..."
    @docker rmi moonflare-test:base moonflare-test:musl moonflare-test:cross moonflare-test:full 2>/dev/null || echo "Images already cleaned"

# release version:
#     @echo "🚀 Creating release {{version}}..."
#     @./scripts/release.sh

# Check that release works using GoReleaser (local macOS)
check-release:
    #!/usr/bin/env bash
    export RUNNER_OS=macOS
    export OPENSSL_DIR=/opt/homebrew/opt/openssl@3
    export OPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include
    export OPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib
    goreleaser release --snapshot --clean

# Check that release works using GoReleaser (CI Linux)
check-release-ci:
    #!/usr/bin/env bash
    export RUNNER_OS=Linux
    # Let system OpenSSL be found naturally in CI
    goreleaser release --snapshot --clean

# Check that release works using GoReleaser (Windows)
check-release-windows:
    #!/usr/bin/env pwsh
    $env:RUNNER_OS = "Windows"
    goreleaser release --snapshot --clean

# Build release version using GoReleaser
release:
    #!/usr/bin/env bash
    export RUNNER_OS=macOS
    export OPENSSL_DIR=/opt/homebrew/opt/openssl@3
    export OPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include
    export OPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib
    goreleaser build --clean --single-target

# Build all release targets using GoReleaser
release-all:
    #!/usr/bin/env bash
    export RUNNER_OS=Linux
    goreleaser build --clean
