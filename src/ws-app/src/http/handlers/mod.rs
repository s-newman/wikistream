use crate::http::server::AppState;
use axum::Router;
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

mod health;
mod index;
mod ingest;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::health))
        .route("/ingest", post(ingest::ingest))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_request(())
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            ),
        )
        .with_state(app_state)
}
