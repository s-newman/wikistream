use anyhow::Context;
use ws_app::db::DbConfig;
use ws_app::http::{HttpServer, HttpServerConfig};
use ws_app::{db, init_tracing, views};

const ENV_VERSION: Option<&'static str> = option_env!("WS_VERSION");
const FALLBACK_VERSION: &str = "Unknown-Version";
// TODO: Use const unwrap_or when stable: https://github.com/rust-lang/rust/issues/143956

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    tracing::info!(
        "starting ws-app {}",
        ENV_VERSION.unwrap_or(FALLBACK_VERSION)
    );

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
