use anyhow::Context;
use ws_app::db::DbConfig;
use ws_app::http::{HttpServer, HttpServerConfig};
use ws_app::{db, init_tracing, version, views};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    tracing::info!("starting ws-app {}", version());

    let db_pool = db::connect(&DbConfig::new())
        .await
        .context("failed to create database connection pool")?;

    let env = views::init()?;

    HttpServer::new(
        HttpServerConfig {
            listen_address: "0.0.0.0:4000".into(),
        },
        db_pool,
        env,
    )
    .await?
    .serve()
    .await
}
