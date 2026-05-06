use crate::PartialEvent;
use anyhow::Context;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use ws_sse::Event;

const EVENTS_PER_FILE: u32 = 10_000;

pub struct EventfileWriter {
    data_dir: PathBuf,
    outfile: Option<BufWriter<File>>,
    event_count: u32,
}

impl EventfileWriter {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().into(),
            outfile: None,
            event_count: 0,
        }
    }

    pub fn save(&mut self, event: Event) -> anyhow::Result<()> {
        // Skip events with empty data
        if event.data.is_empty() {
            return Ok(());
        }

        // TODO: replace with get_or_insert_with when stabilized
        // ref: https://github.com/rust-lang/rust/issues/143648
        let writer = match &mut self.outfile {
            Some(x) => x,
            None => match self.start_new_file(&event)? {
                Some(y) => y,
                None => {
                    // We have to skip the event because we can't parse a timestamp from it for the
                    // new file :(
                    // The event data has been logged though, so we may be able to manually backfill
                    // it into the event files.
                    return Ok(());
                }
            },
        };

        writer
            .write_all(event.data.as_bytes())
            .context("error writing event data to file")?;
        writer
            .write_all(b"\n")
            .context("error writing newline to file")?;

        self.event_count += 1;

        // Rotate to next file after a set number of lines (setting self.outfile to None will make
        // us open a new
        if self.event_count.is_multiple_of(EVENTS_PER_FILE)
            && let Some(mut writer) = self.outfile.take()
            && let Err(e) = writer.flush()
        {
            // If we haven't flushed everything to disk, we need to quit to prevent
            // gaps in the data (the EventReader will try to continue after an event
            // ID that wasn't written to disk).
            Err(e).context("error flushing buffer to disk during file rotation")?
        }

        Ok(())
    }

    fn start_new_file(&mut self, event: &Event) -> anyhow::Result<Option<&mut BufWriter<File>>> {
        let Ok(partial_evt) = serde_json::from_str::<PartialEvent>(&event.data) else {
            tracing::warn!(
                data = &event.data,
                "failed to parse event data as partial event"
            );
            return Ok(None);
        };
        let timestamp = partial_evt.meta.dt.timestamp_millis();
        let fname = self.data_dir.join(format!("events-{timestamp}.jsonl"));
        let file = File::create(&fname).context("error opening new event file")?;
        tracing::info!(filename = %fname.display(), "starting new output file");

        Ok(Some(self.outfile.insert(BufWriter::new(file))))
    }
}
