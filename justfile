default:
    @brew install git unzip gzip xz
    @proto pin node stable --to local
    @proto pin rust stable --to local
    @moon setup

create-project dest tmpl example_dir:
    @curl -L -o repo.zip https://github.com/cloudflare/templates/archive/refs/heads/main.zip && unzip repo.zip "templates-main/{{ example_dir }}/*" && mv templates-main/{{ example_dir }} {{ dest }} && rm -r templates-main repo.zip
    @moon generate {{ tmpl }} {{ dest }}

create-astro-app dest:
    @just create-project {{ dest }} astro-moon astro-blog-starter-template

create-react-app dest:
    @just create-project {{ dest }} react-moon vite-react-template

create-durable-object dest:
    @just create-project {{ dest }} durable-object-moon hello-world-do-template

create-crate dest:
    @cargo new --vcs none --bin --edition 2024 {{ dest }}
