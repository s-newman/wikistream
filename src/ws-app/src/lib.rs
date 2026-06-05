pub mod db;
pub mod http;
mod version;
pub mod views;

pub use db::Pool as DbPool;
pub use version::version;

pub fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_target(false)
        .compact()
        .init();
}

/// Get a router using the provided database connection.
///
/// This is used in integration tests to get an instance of the application to
/// run end-to-end tests against without requiring the application to listen on
/// a port.
pub fn get_test_router(db: sqlx::PgPool) -> axum::Router {
    http::handlers::router(http::server::AppState {
        db_pool: db,
        env: std::sync::Arc::new(views::init().unwrap()),
    })
}
