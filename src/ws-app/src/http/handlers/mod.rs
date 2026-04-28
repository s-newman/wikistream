use crate::http::server::AppState;
use axum::Router;
use axum::routing::{get, post};

mod health;
mod index;
mod ingest;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::health))
        .route("/ingest", post(ingest::ingest))
        .with_state(app_state)
}
