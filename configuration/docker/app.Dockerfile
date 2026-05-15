FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN ls && cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --package ws-app --release

FROM debian:stable-slim AS runtime
WORKDIR /var/www
VOLUME /var/local/ws-app
EXPOSE 4000
COPY --from=builder /app/target/release/ws-app /usr/local/bin
RUN mkdir /var/www/assets
COPY --from=builder /app/src/ws-app/assets /var/www/assets
RUN mkdir /var/www/templates
COPY --from=builder /app/src/ws-app/templates /var/www/templates
ENTRYPOINT ["/usr/local/bin/ws-app"]
