use crate::http::server::AppState;
use axum::Router;
use axum::routing::{get, post};
use tower_http::services::ServeDir;

mod health;
mod index;
mod ingest;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::health))
        .route("/ingest", post(ingest::ingest))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(app_state)
}
