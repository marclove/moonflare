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
    @cargo clippy

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

test-fast:
    @cargo test --bins

# Cross-platform build tasks
build-target target:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🔄 Building for target: {{target}}"
    if [[ "{{target}}" == "x86_64-unknown-linux-gnu" ]] || [[ "$RUNNER_OS" != "Linux" ]]; then
        cargo build --release --target {{target}}
    else
        cross build --release --target {{target}}
    fi
    echo "✅ Build complete for {{target}}"

# Package release artifacts  
package-release target name version archive_type:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "📦 Packaging {{name}} for {{version}}"
    
    mkdir -p release
    cd target/{{target}}/release
    
    if [[ "{{target}}" == *"windows"* ]]; then
        BINARY_NAME="moonflare.exe"
    else
        BINARY_NAME="moonflare"
        strip moonflare
    fi
    
    if [[ "{{archive_type}}" == "zip" ]]; then
        7z a ../../../release/{{name}}-{{version}}.zip $BINARY_NAME
        cd ../../../release
        sha256sum {{name}}-{{version}}.zip > {{name}}-{{version}}.zip.sha256
    else
        tar -czf ../../../release/{{name}}-{{version}}.tar.gz $BINARY_NAME
        cd ../../../release  
        sha256sum {{name}}-{{version}}.tar.gz > {{name}}-{{version}}.tar.gz.sha256
    fi
    
    echo "✅ Package created: {{name}}-{{version}}.{{archive_type}}"

release-prep: cli-fmt cli-lint cli-test
    @echo "🔍 Checking if working directory is clean..."
    @git diff --quiet || (echo "❌ Working directory is not clean. Commit changes first." && exit 1)
    @echo "✅ Ready for release"

release version:
    @echo "🚀 Creating release {{version}}..."
    @./scripts/release.sh
