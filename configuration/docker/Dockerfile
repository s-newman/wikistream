FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN ls && cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --package ws-sse-cli --release

FROM debian:stable-slim AS runtime
VOLUME /var/local/ws-sse-cli
COPY --from=builder /app/target/release/ws-sse-cli /usr/local/bin
ENTRYPOINT ["/usr/local/bin/ws-sse-cli", "--data-dir", "/var/local/ws-sse-cli"]