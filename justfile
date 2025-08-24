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
