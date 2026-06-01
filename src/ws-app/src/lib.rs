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
