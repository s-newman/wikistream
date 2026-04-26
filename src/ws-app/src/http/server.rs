use crate::DbPool;
use crate::http::handlers;
use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;

pub struct HttpServerConfig {
    pub listen_address: String,
}

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    pub async fn new(config: HttpServerConfig, db_pool: DbPool) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(&config.listen_address)
            .await
            .with_context(|| format!("failed to listen on address '{}'", &config.listen_address))?;

        let router = handlers::router(db_pool);

        Ok(Self { router, listener })
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        Ok(axum::serve(self.listener, self.router).await?)
    }
}
