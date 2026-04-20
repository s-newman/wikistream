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

# Remove build artifacts
clean:
    cargo clean --workspace

# Run ws-sse-cli
run *ARGS:
    cargo run --package ws-sse-cli -- {{ARGS}}

# Build a docker container to run ws-sse-cli
docker-build:
    docker build -t ws-sse-cli:latest -f configuration/docker/Dockerfile .

# Run ws-sse-cli in a docker container
docker-run: docker-build
    docker run --rm -d -v "$(pwd)/data:/var/local/ws-sse-cli" ws-sse-cli:latest