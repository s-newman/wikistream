mod eventfile;
mod ingest;

use crate::eventfile::EventfileWriter;
use crate::ingest::IngestClientBuilder;
use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;
use std::{fs, io};
use tracing::Level;
use ws_models::Event;
use ws_sse::EventSource;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    limit: u32,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Consume events from EventStreams and save them to disk.
    Stream {
        /// Event ID to resume from.
        #[arg(short, long)]
        event_id: Option<String>,
    },
    /// Read saved events from disk.
    ///
    /// Currently only used to verify the contents of the files stored on disk.
    Read,
    /// Ingest saved files from disk into an app instance.
    Ingest {
        #[arg(long, short, default_value = "http://localhost:4000")]
        server: String,
    },
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

    if let Err(e) = match args.command {
        Command::Stream { event_id } => stream(&args.data_dir, event_id),
        Command::Read => read(&args.data_dir, args.limit),
        Command::Ingest { server } => ingest(&args.data_dir, server),
    } {
        tracing::error!(error = ?e, "unexpected error");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn ingest(data_dir: &Path, server: String) -> anyhow::Result<()> {
    let data_files = get_data_files(data_dir).context("failed to read data dir")?;
    let mut ingest_client = IngestClientBuilder::new().with_server(&server).build();

    for data_file in data_files {
        tracing::info!(filename = %data_file.display(), "ingesting new data file");
        let ib = BufReader::new(File::open(&data_file).context("failed to open data file")?);
        for (idx, line) in ib.lines().enumerate() {
            let line = line.with_context(|| {
                    tracing::error!(lineno = idx + 1, filename=%data_file.to_string_lossy(), "failed to read line");
                    "failed to read line"
                })?;
            ingest_client
                .ingest(line)
                .context("failed to ingest line")?;
        }

        ingest_client
            .flush()
            .context("failed to flush ingest queue")?;
    }

    Ok(())
}

#[expect(dead_code)]
fn timestamp_from_file(file: &Path) -> anyhow::Result<i64> {
    let Some(filename) = file.file_name() else {
        bail!("no file name");
    };
    let filename = filename.to_string_lossy();
    let Some(end) = filename.strip_prefix("events-") else {
        bail!("wrong filename prefix");
    };
    let Some(timestamp) = end.strip_suffix(".jsonl") else {
        bail!("wrong filename suffix");
    };

    i64::from_str(timestamp).context("failed to parse filename part as timestamp")
}

fn read(data_dir: &Path, limit: u32) -> anyhow::Result<()> {
    let data_files = get_data_files(data_dir).context("failed to read data dir")?;

    let mut total_parsed = 0u64;
    for data_file in data_files {
        tracing::info!(filename = %data_file.display(), "reading new data file");
        let ib = BufReader::new(File::open(&data_file).context("failed to open data file")?);
        for (idx, line) in ib.lines().enumerate() {
            let line = line.with_context(|| {
                tracing::error!(lineno = idx + 1, filename=%data_file.to_string_lossy(), "failed to read line");
                "failed to read line"
            })?;
            serde_json::from_str::<Event>(&line).with_context(|| {
                tracing::error!(lineno = idx + 1, filename=%data_file.to_string_lossy(), "failed to parse line as json");
                "failed to parse line as json"
            })?;
            total_parsed += 1;

            if limit > 0 && total_parsed > limit.into() {
                break;
            }
        }
    }

    Ok(())
}

fn stream(data_dir: &Path, event_id: Option<String>) -> anyhow::Result<()> {
    let mut writer = EventfileWriter::new(data_dir);
    let event_id = match event_id {
        None => get_event_id_from_files(data_dir)
            .context("failed to get latest event ID from data files")?,
        x => x,
    };

    let es = EventSource::new("https://stream.wikimedia.org/v2/stream/recentchange")
        .with_event_id(event_id);

    for x in es {
        match x {
            Ok(event) => writer.save(event).context("failed to save event to disk")?,
            Err(e) => {
                tracing::error!(error = %e, "unexpected error when streaming events");
                bail!("quitting due to fatal error");
            }
        }
    }

    Ok(())
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
