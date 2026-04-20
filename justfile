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