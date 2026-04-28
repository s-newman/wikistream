pub mod db;
pub mod http;
pub mod views;

pub use db::Pool as DbPool;

use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
