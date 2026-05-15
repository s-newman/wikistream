mod eventfile;
mod ingest;

use crate::eventfile::EventfileWriter;
use crate::ingest::{IngestClient, IngestClientBuilder};
use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::mpsc::TrySendError;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io, thread};
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

        #[arg(long, short, default_value = "http://localhost:4000")]
        server: String,
    },
    /// Read saved events from disk.
    ///
    /// Currently only used to verify the contents of the files stored on disk.
    Read,
    /// Ingest saved files from disk into an app instance.
    Ingest {
        #[arg(long, short, default_value = "http://localhost:4000")]
        server: String,

        /// If specified, start ingesting events from this time or later.
        #[arg(long)]
        start: Option<String>,

        /// If specified, don't ingest any events after this time.
        #[arg(long)]
        end: Option<String>,
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
        Command::Stream { event_id, server } => stream(&args.data_dir, event_id, &server),
        Command::Read => read(&args.data_dir, args.limit),
        Command::Ingest { server, start, end } => {
            // TODO: more ergonomic date/time specification, better error reporting
            let start = start.map(parse_date_arg).transpose().unwrap();
            let end = end.map(parse_date_arg).transpose().unwrap();
            ingest(&args.data_dir, server, start, end)
        }
    } {
        tracing::error!(error = ?e, "unexpected error");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn parse_date_arg(arg: String) -> anyhow::Result<DateTime<Utc>> {
    if let Ok(dt) = arg.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }

    if let Ok(dt) = arg.parse::<NaiveDateTime>() {
        return Ok(dt.and_utc());
    }

    if let Ok(dt) = arg.parse::<NaiveDate>() {
        return Ok(NaiveDateTime::from(dt).and_utc());
    }

    bail!("failed to parse arg as date");
}

fn ingest(
    data_dir: &Path,
    server: String,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    let data_files = get_data_files(data_dir).context("failed to read data dir")?;
    let mut ingest_client = IngestClientBuilder::new().with_server(&server).build();

    // Find first data file if --start specified (should be the last file with a timestamp before
    // the --start timestamp)
    let mut start_idx = 0_usize;
    if let Some(start) = start {
        for (idx, data_file) in data_files.iter().enumerate() {
            if timestamp_from_file(data_file).context("failed to read timestamp from filename")?
                < start
            {
                start_idx = idx;
            } else {
                break;
            }
        }
    }

    // Find the last data file if --end specified (same thing, looking for the first file with a
    // timestamp after the --end timestamp)
    let mut end_idx = data_files.len() - 1;
    if let Some(end) = end {
        for (idx, data_file) in data_files.iter().enumerate() {
            // TODO: this is inefficient, we're re-parsing the timestamps from every filename
            if timestamp_from_file(data_file).context("failed to read timestamp from filename")?
                > end
            {
                end_idx = idx;
                break;
            }
        }
    }

    for data_file in &data_files[start_idx..=end_idx] {
        tracing::info!(filename = %data_file.display(), "ingesting new data file");
        let ib = BufReader::new(File::open(data_file).context("failed to open data file")?);
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

fn timestamp_from_file(file: &Path) -> anyhow::Result<DateTime<Utc>> {
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

    let millis = i64::from_str(timestamp).context("failed to parse filename part as timestamp")?;

    DateTime::from_timestamp_millis(millis)
        .ok_or_else(|| anyhow!("timestamp from filename was out of bounds: {}", millis))
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

/// When streaming, we want to ingest event batches to the server more
/// frequently, so we'll use smaller batches.
const STREAM_BATCH_SIZE: usize = 100;

/// Arbitrarily-selected max size for ingest worker queue.
///
/// Temporary increase to reduce the frequency we get a full queue. Need to do some analysis to
/// figure out an appropriate upper bound.
const INGEST_WORKER_QUEUE_SIZE: usize = STREAM_BATCH_SIZE * 10;

enum IngestWorkerCmd {
    Line(String),
    Quit,
}

fn stream(data_dir: &Path, event_id: Option<String>, server: &str) -> anyhow::Result<()> {
    let mut writer = EventfileWriter::new(data_dir);
    let ingest_client = IngestClientBuilder::new()
        .with_server(server)
        .with_batch_size(STREAM_BATCH_SIZE)
        .build();
    // Using a bounded channel to send raw events to the ingest client, so if
    // there are problems sending data we don't try to queue an unbounded
    // number of raw events in the channel.
    //
    // This means we're more likely to drop events instead of ingest them, but
    // we can catch those with regularly-scheduled backfills and monitoring.
    let (tx, rx) = sync_channel::<IngestWorkerCmd>(INGEST_WORKER_QUEUE_SIZE);
    let handle = thread::spawn(move || stream_ingest_worker(rx, ingest_client));
    let event_id = match event_id {
        None => get_event_id_from_files(data_dir)
            .context("failed to get latest event ID from data files")?,
        x => x,
    };

    let es = EventSource::new("https://stream.wikimedia.org/v2/stream/recentchange")
        .with_event_id(event_id);

    let inner_result = stream_loop(es, &mut writer, &tx);

    if let Err(e) = tx.try_send(IngestWorkerCmd::Quit) {
        tracing::error!(error = %e, "failed to send quit command to ingest worker");
    }

    if !handle.is_finished() {
        // TODO: this is a very arbitrary amount of time to wait. need to check timeouts on the
        // ingest client to find a more appropriate duration to wait for the ingest worker to
        // cleanly exit
        sleep(Duration::from_secs(3));
    }

    if handle.is_finished() {
        match handle.join() {
            Ok(Err(e)) => tracing::error!(error = %e, "ingest worker exited with error"),
            Err(e) => {
                // ref: https://users.rust-lang.org/t/std-thread-join-return-type/63395/4
                let err_msg = match (e.downcast_ref::<&str>(), e.downcast_ref::<String>()) {
                    (Some(&s), _) => s,
                    (_, Some(s)) => s,
                    (None, None) => "<No panic info>",
                };
                tracing::error!("ingest worker thread panicked with: {}", err_msg);
            }
            _ => (),
        }
    } else {
        tracing::error!("ingest worker thread is not finished - it will be orphaned!");
    }

    inner_result
}

fn stream_loop(
    es: EventSource,
    writer: &mut EventfileWriter,
    tx: &SyncSender<IngestWorkerCmd>,
) -> anyhow::Result<()> {
    for x in es {
        match x {
            Ok(event) => {
                writer
                    .save(&event)
                    .context("failed to save event to disk")?;
                match tx.try_send(IngestWorkerCmd::Line(event.data)) {
                    Ok(_) => (),
                    Err(TrySendError::Full(_)) => {
                        // TODO: metrics
                        tracing::warn!("could not ingest event because worker queue was full");
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        tracing::error!("ingest worker has disconnected from queue");
                        bail!("ingest worker disconnected from queue");
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

fn stream_ingest_worker(
    rx: Receiver<IngestWorkerCmd>,
    mut ingest_client: IngestClient,
) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(IngestWorkerCmd::Line(line)) => {
                if let Err(e) = ingest_client.ingest(line) {
                    tracing::error!(error = ?e, "ingest worker failed to queue event for ingest");
                    return Err(e).context("ingest worker failed to queue event for ingest");
                }
            }
            Ok(IngestWorkerCmd::Quit) => return ingest_client.flush(),
            Err(e) => {
                // Logging now because it might be a while before the parent
                // process joins the handle and discovers the error
                tracing::error!(error = %e, "ingest worker failed to receive from the channel");
                return Err(e).context("ingest worker failed to receive from the channel")?;
            }
        }
    }
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
