use crate::DbPool;
use axum::Router;
use axum::routing::{get, post};

mod health;
mod ingest;

pub fn router(db_pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ingest", post(ingest::ingest))
        .with_state(db_pool)
}
