set dotenv-load

# Build and check (run me before committing)
default: build check

# Build
build *ARGS:
    scripts/build.sh {{ARGS}}

# Run linters
check:
    scripts/check.sh

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
    cargo run --package ws-sse-cli -- {{ARGS}}

# Run ws-app
[working-directory: 'src/ws-app']
run-app *ARGS:
    cargo run --package ws-app -- {{ARGS}}

# Build all docker containers
docker-build: docker-build-app docker-build-cli

# Build a docker container to run ws-app
docker-build-app:
    docker build -t ws-app:latest -f configuration/docker/app.Dockerfile .

# Build a docker container to run ws-sse-cli
docker-build-cli:
    docker build -t ws-sse-cli:latest -f configuration/docker/cli.Dockerfile .

# Run ws-app in a docker container
docker-run-app: docker-build-app
    docker run \
        --restart unless-stopped \
        -d \
        -p 80:4000 \
        --network docker_default \
        --env-file .env \
        -e PGHOST=db \
        ws-app:latest

# Run ws-sse-cli in a docker container
docker-run-cli: docker-build-cli
    docker run \
        --restart unless-stopped \
        -d \
        -v "$(pwd)/data:/var/local/ws-sse-cli" \
        ws-sse-cli:latest \
        stream --server http://wikistream.altoidtin.com

# Start a local Postgres database in a docker container for development
db-up:
    docker compose -f configuration/docker/db.compose.yaml up -d

# Stop the development database
db-down:
    docker compose -f configuration/docker/db.compose.yaml down -v
