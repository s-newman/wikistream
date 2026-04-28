use crate::DbPool;
use crate::http::handlers;
use anyhow::Context;
use axum::Router;
use minijinja::Environment;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct HttpServerConfig {
    pub listen_address: String,
}

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub env: Arc<Environment<'static>>,
}

impl HttpServer {
    pub async fn new(
        config: HttpServerConfig,
        db_pool: DbPool,
        env: Environment<'static>,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(&config.listen_address)
            .await
            .with_context(|| format!("failed to listen on address '{}'", &config.listen_address))?;
        let local_addr = listener
            .local_addr()
            .context("failed to get local address")?;
        tracing::info!("listenining on {}", local_addr);

        let router = handlers::router(AppState {
            db_pool,
            env: Arc::new(env),
        });

        Ok(Self { router, listener })
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        Ok(axum::serve(self.listener, self.router).await?)
    }
}
