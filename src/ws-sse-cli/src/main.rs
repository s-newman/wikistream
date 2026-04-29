mod ingest_state;

use crate::ingest_state::IngestState;
use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use crossbeam::channel::{Receiver, RecvTimeoutError, SendTimeoutError, Sender, bounded};
use crossbeam::sync::WaitGroup;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, io, thread};
use tracing::Level;
use ureq::Agent;
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

        #[arg(long, default_value_t = 10_000)]
        events_per_file: u32,
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
        Command::Stream {
            event_id,
            events_per_file,
        } => stream(&args.data_dir, args.limit, event_id, events_per_file),
        Command::Read => read(&args.data_dir, args.limit),
        Command::Ingest { server } => ingest(&args.data_dir, args.limit, server),
    } {
        tracing::error!(error = ?e, "unexpected error");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

const UPDATE_STATE_INTERVAL: u64 = 1000;
const QUEUE_CAP: usize = 100;
const WORKERS: usize = 16;

fn ingest(data_dir: &Path, limit: u32, server: String) -> anyhow::Result<()> {
    let mut data_files = get_data_files(data_dir).context("failed to read data dir")?;
    let ingest_endpoint = format!("{}/ingest", server);
    let previous_progress = IngestState::load(data_dir).context("failed to load ingest state")?;

    let wg = WaitGroup::new();
    let (tx, rx) = bounded::<String>(QUEUE_CAP);
    let exit = Arc::new(AtomicBool::new(false));
    let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
    while join_handles.len() < WORKERS {
        let w_rx = rx.clone();
        let w_wg = wg.clone();
        let w_exit = exit.clone();
        let w_endpoint = ingest_endpoint.clone();
        join_handles.push(thread::spawn(move || {
            ingest_worker(w_rx, w_wg, w_exit, w_endpoint)
        }));
    }

    if let Some(progress) = previous_progress {
        if let Some(newer_file_idx) = data_files.iter().position(|x| {
            let file_ts = match timestamp_from_file(x) {
                Ok(ts) => ts,
                Err(e) => {
                    tracing::error!(error = %e, filename = %x.display(), "BUG: failed to parse timestamp from file");
                    panic!("failed to parse timestamp from file");
                }
            };
            file_ts > progress.last_ingested_ms
        }) {
            // If we found a file newer than the progress timestamp, resume with at most two files
            // before that
            let resume_file_idx = newer_file_idx.saturating_sub(2);
            let split = data_files.split_off(resume_file_idx);
            data_files = split;
        } else {
            // The timestamps in all filenames we found are older than the saved progress. Go ahead
            // and re-ingest the most recent three files before that.
            let resume_file_idx = data_files.len() - 3;
            let split = data_files.split_off(resume_file_idx);
            data_files = split;
        }
    }

    let mut total_ingested = 0u64;
    for data_file in data_files {
        tracing::info!(filename = %data_file.display(), "ingesting new data file");
        let ib = BufReader::new(File::open(&data_file).context("failed to open data file")?);
        for (idx, line) in ib.lines().enumerate() {
            let line = line.with_context(|| {
                    tracing::error!(lineno = idx + 1, filename=%data_file.to_string_lossy(), "failed to read line");
                    "failed to read line"
                })?;
            let line_clone = if total_ingested.is_multiple_of(UPDATE_STATE_INTERVAL) {
                Some(line.clone())
            } else {
                None
            };
            send_until(line, &tx, &exit)?;
            total_ingested += 1;

            if let Some(line) = line_clone {
                let Ok(partial_evt) = serde_json::from_str::<PartialEvent>(&line) else {
                    tracing::warn!(data = &line, "failed to parse event data as partial event");
                    continue;
                };
                let ingest_state = IngestState {
                    last_ingested_ms: partial_evt.meta.dt.timestamp_millis(),
                };
                if let Err(e) = ingest_state.save(data_dir) {
                    tracing::error!(error = %e, "failed to record ingest progress");
                }
            }

            if limit > 0 && total_ingested > limit.into() {
                return Ok(());
            }
        }
    }

    Ok(())
}

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

fn send_until<T>(mut msg: T, tx: &Sender<T>, exit: &Arc<AtomicBool>) -> anyhow::Result<()> {
    while !exit.load(Ordering::Relaxed) {
        match tx.send_timeout(msg, Duration::from_millis(100)) {
            Ok(_) => return Ok(()),
            Err(SendTimeoutError::Timeout(x)) => {
                msg = x;
                continue;
            }
            Err(SendTimeoutError::Disconnected(_)) => {
                bail!("failed to send over disconnected work channel")
            }
        }
    }

    Ok(())
}

fn ingest_worker(rx: Receiver<String>, wg: WaitGroup, exit: Arc<AtomicBool>, endpoint: String) {
    if let Err(e) = ingest_worker_inner(rx, exit.clone(), endpoint) {
        tracing::error!(error = ?e, "worker failed with error");
        exit.swap(true, Ordering::Relaxed);
    }
    drop(wg)
}

fn ingest_worker_inner(
    rx: Receiver<String>,
    exit: Arc<AtomicBool>,
    endpoint: String,
) -> anyhow::Result<()> {
    let agent: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();

    while !exit.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(x) => {
                let resp = agent
                    .post(&endpoint)
                    .send(x)
                    .context("failed to send request")?;
                if resp.status() != StatusCode::OK && resp.status() != StatusCode::CONFLICT {
                    bail!("server returned bad status code: {}", resp.status());
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => bail!("work channel disconnected"),
        }
    }

    Ok(())
}

fn read(data_dir: &Path, limit: u32) -> anyhow::Result<()> {
    let data_files = get_data_files(data_dir).context("failed to read data dir")?;

    let mut total_parsed = 0u64;
    for data_file in data_files {
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

fn stream(
    data_dir: &Path,
    limit: u32,
    event_id: Option<String>,
    events_per_file: u32,
) -> anyhow::Result<()> {
    let event_id = match event_id {
        None => get_event_id_from_files(data_dir)
            .context("failed to get latest event ID from data files")?,
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
                        let fname = data_dir.join(format!("events-{timestamp}.jsonl"));
                        let file = File::create(&fname).context("error opening new event file")?;
                        tracing::info!(filename = %fname.display(), "starting new output file");

                        BufWriter::new(file)
                    }
                };

                writer
                    .write_all(event.data.as_bytes())
                    .context("error writing event data to file")?;
                writer
                    .write_all(b"\n")
                    .context("error writing newline to file")?;
                {
                    outfile = Some(writer);

                    event_count += 1;
                    if limit > 0 && event_count >= limit {
                        break;
                    }

                    // Rotate to next file after a set number of lines
                    if event_count % events_per_file == 0
                        && let Some(mut writer) = outfile.take()
                        && let Err(e) = writer.flush()
                    {
                        // If we haven't flushed everything to disk, we need to quit to prevent
                        // gaps in the data (the EventReader will try to continue after an event
                        // ID that wasn't written to disk).
                        Err(e).context("error flushing buffer to disk during file rotation")?
                    }
                }
            }
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
