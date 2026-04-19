use std::io;
use std::process::ExitCode;
use tracing::Level;
use ws_sse::EventSource;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .init();

    let es = EventSource::new("https://stream.wikimedia.org/v2/stream/recentchange");

    for x in es {
        match x {
            Ok(event) => println!("{}", event.data),
            Err(e) => {
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
