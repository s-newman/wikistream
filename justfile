set dotenv-load := true

# Set version string for commands like `cargo run` that don't run the
# get-tag.sh script

export WS_VERSION := `scripts/get-tag.sh`

# Build and check (run me before committing)
default: build check

# Build
build *ARGS:
    scripts/build.sh {{ ARGS }}

# Run linters & tests
check: lint test

# Run linters
lint:
    scripts/check.sh

# Run tests
test:
    cargo test --workspace

# Reformat code
format:
    cargo fmt --all

alias fmt := format

# Fix linting issues caught by clippy
fix:
    cargo clippy --no-deps --all-targets --fix -- -D warnings

# Remove build artifacts
clean:
    cargo clean --workspace

# Run ws-sse-cli
run *ARGS:
    cargo run --package ws-sse-cli -- {{ ARGS }}

# Run ws-app
[working-directory('src/ws-app')]
run-app *ARGS:
    cargo run --package ws-app -- {{ ARGS }}

# Build all docker containers
docker-build: docker-build-app docker-build-cli

# Build a docker container to run ws-app
docker-build-app:
    scripts/docker-build.sh ws-app app.Dockerfile

# Build a docker container to run ws-sse-cli
docker-build-cli:
    scripts/docker-build.sh ws-sse-cli cli.Dockerfile

# Run ws-app in a docker container
docker-run-app: docker-build-app
    #!/bin/bash
    tag="$(docker image inspect ws-app:latest | jq -r '.[0].RepoTags[]' | grep -v ':latest')"
    docker stop ws-app || true
    docker rm ws-app || true
    docker run \
        --restart unless-stopped \
        -d \
        -p 80:4000 \
        --network docker_default \
        --env-file .env \
        -e PGHOST=db \
        --name ws-app \
        "${tag}"

# Run ws-sse-cli in a docker container
docker-run-cli: docker-build-cli
    #!/bin/bash
    tag="$(docker image inspect ws-sse-cli:latest | jq -r '.[0].RepoTags[]' | grep -v ':latest')"
    docker stop ws-sse-cli || true
    docker rm ws-sse-cli || true
    docker run \
        --restart unless-stopped \
        -d \
        -v "$(pwd)/data:/var/local/ws-sse-cli" \
        --name ws-sse-cli \
        "${tag}" \
        stream --server http://wikistream.altoidtin.com

# Start a local Postgres database in a docker container for development
db-up:
    docker compose -f configuration/docker/db.compose.yaml up -d

# Stop the development database
db-down:
    docker compose -f configuration/docker/db.compose.yaml down -v
