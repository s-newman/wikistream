pub mod db;
pub mod http;
pub mod views;

pub use db::Pool as DbPool;

pub fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_target(false)
        .compact()
        .init();
}
