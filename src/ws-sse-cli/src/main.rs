use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, io};
use tracing::Level;
use ws_sse::EventSource;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Event ID to resume from.
    #[arg(short, long)]
    event_id: Option<String>,

    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    limit: u32,

    #[arg(long, default_value_t = 10_000)]
    events_per_file: u32,
}

fn main() -> ExitCode {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .init();

    if !args.data_dir.exists()
        && let Err(e) = fs::create_dir_all(&args.data_dir)
    {
        tracing::error!(error = %e, "failed to create data directory");
        return ExitCode::FAILURE;
    }

    let event_id = match args.event_id {
        None => match get_event_id_from_files(&args.data_dir) {
            Ok(x) => x,
            Err(e) => {
                tracing::error!(error = ?e, "failed to get latest event ID from data files");
                return ExitCode::FAILURE;
            }
        },
        x => x,
    };

    let es = EventSource::new("https://stream.wikimedia.org/v2/stream/recentchange")
        .with_event_id(event_id);

    let mut outfile: Option<BufWriter<File>> = None;

    let mut event_count = 0;
    for x in es {
        match x {
            Ok(event) => {
                // skip events with empty data
                if event.data.is_empty() {
                    continue;
                }

                // if no outfile is open, parse the object to get the timestamp so we can put the
                // timestamp in the output filename
                // TODO: replace with get_or_insert_with when stabilized
                // ref: https://github.com/rust-lang/rust/issues/143648
                let mut writer = match outfile {
                    Some(wr) => wr,
                    None => {
                        let Ok(partial_evt) = serde_json::from_str::<PartialEvent>(&event.data)
                        else {
                            tracing::warn!(
                                data = &event.data,
                                "failed to parse event data as partial event"
                            );
                            continue;
                        };
                        let timestamp = partial_evt.meta.dt.timestamp_millis();
                        let fname = args.data_dir.join(format!("events-{timestamp}.jsonl"));
                        let file = match File::create(&fname) {
                            Ok(x) => x,
                            Err(e) => {
                                tracing::error!(error = ?e, filename = %fname.display(), "error opening new output file");
                                return ExitCode::FAILURE;
                            }
                        };

                        tracing::info!(filename = %fname.display(), "starting new output file");

                        BufWriter::new(file)
                    }
                };

                if let Err(e) = writer.write_all(event.data.as_bytes()) {
                    tracing::error!(error = ?e, "error writing event data to file");
                    return ExitCode::FAILURE;
                }
                if let Err(e) = writer.write_all(b"\n") {
                    tracing::error!(error = ?e, "error writing newline to file");
                    return ExitCode::FAILURE;
                }

                outfile = Some(writer);

                event_count += 1;
                if args.limit > 0 && event_count >= args.limit {
                    break;
                }

                // Rotate to next file after a set number of lines
                if event_count % args.events_per_file == 0
                    && let Some(mut writer) = outfile.take()
                    && let Err(e) = writer.flush()
                {
                    tracing::error!(error = ?e, "error flushing buffer to disk during file rotation");
                    // If we haven't flushed everything to disk, we need to quit to prevent
                    // gaps in the data (the EventReader will try to continue after an event
                    // ID that wasn't written to disk).
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "unexpected error when streaming events");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

#[derive(Deserialize)]
struct PartialEvent {
    meta: PartialMeta,
}

#[derive(Deserialize)]
struct PartialMeta {
    topic: String,
    partition: i64,
    dt: DateTime<Utc>,
}

#[derive(Serialize)]
struct KafkaPtr {
    topic: String,
    partition: i64,
    timestamp: i64,
}

fn get_event_id_from_files(data_dir: &Path) -> anyhow::Result<Option<String>> {
    let files = get_data_files(data_dir).context("error reading data directory")?;

    let newest = match files.last() {
        Some(x) => x,
        None => return Ok(None),
    };

    let file = File::open(newest).context("error opening latest data file")?;
    let reader = BufReader::new(file);

    // This will read the entire file instead of trying to read backwards in chunks, which is more
    // efficient for large files but also more complicated. The event files we're reading are only
    // tens of MiBs, so we're sticking with the inefficient but simple method because the perf
    // doesn't really matter.
    let Some(last_event) = reader
        .lines()
        .map_while(Result::ok)
        // Convert to partial event, only keep lines that successfully parsed
        .filter_map(|l| serde_json::from_str::<PartialEvent>(&l).ok())
        .last()
    else {
        return Ok(None);
    };

    let id_list = vec![KafkaPtr {
        topic: last_event.meta.topic,
        partition: last_event.meta.partition,
        timestamp: last_event.meta.dt.timestamp_millis(),
    }];

    Ok(Some(
        serde_json::to_string(&id_list).context("error serializing event ID to string")?,
    ))
}

fn get_data_files(data_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for file in data_dir.read_dir()? {
        let file = file?;
        if file.metadata()?.len() == 0 {
            // skip empty files
            continue;
        }

        let _fname = file.file_name(); // separate let required to avoid E0716
        let fname = _fname.to_string_lossy();
        if fname.starts_with("events-") && fname.ends_with(".jsonl") {
            files.push(file.path());
        }
    }

    files.sort();
    Ok(files)
}
