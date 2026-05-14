use anyhow::{Context, bail};
use http::StatusCode;
use std::io::{BufWriter, Write};
use std::str::FromStr;
use ureq::Agent;
use ws_models::{Event, FullEvent};

const DEFAULT_BATCH_SIZE: usize = 1000;
const DEFAULT_ENDPOINT: &str = "http://localhost/ingest";

pub struct IngestClientBuilder {
    client: Agent,
    endpoint: String,
    batch_size: usize,
}

impl Default for IngestClientBuilder {
    fn default() -> Self {
        let client: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            client,
            endpoint: DEFAULT_ENDPOINT.into(),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
}

impl IngestClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_server(self, server: &str) -> Self {
        Self {
            endpoint: format!("{}/ingest", server),
            ..self
        }
    }

    pub fn with_batch_size(self, batch_size: usize) -> Self {
        Self { batch_size, ..self }
    }

    pub fn build(self) -> IngestClient {
        IngestClient {
            client: self.client,
            event_buffer: Vec::with_capacity(self.batch_size),
            endpoint: self.endpoint,
            batch_size: self.batch_size,
        }
    }
}

pub struct IngestClient {
    client: Agent,
    event_buffer: Vec<Event>,
    endpoint: String,
    batch_size: usize,
}

impl IngestClient {
    /// Queue an event for ingest.
    pub fn ingest(&mut self, line: String) -> anyhow::Result<()> {
        // Skip events with empty data
        // The first event received from EventStreams upon connection will always be empty
        if line.is_empty() {
            return Ok(());
        }

        let event = Event::from_str(&line).context("failed to parse line as event")?;

        let Event::Event(FullEvent::Edit(edit_event)) = &event else {
            return Ok(());
        };
        if edit_event.shared.wiki != "enwiki" {
            return Ok(());
        }

        self.event_buffer.push(event);

        if self.event_buffer.len() >= self.batch_size {
            self.ingest_batch()
        } else {
            Ok(())
        }
    }

    /// Ingest any queued events.
    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.ingest_batch()
    }

    fn ingest_batch(&mut self) -> anyhow::Result<()> {
        let mut req_body = BufWriter::new(Vec::new());
        for event in self.event_buffer.iter() {
            serde_json::to_writer(&mut req_body, event).context("failed to serialize event")?;
            req_body
                .write_all(b"\n")
                .context("failed to write newline")?;
        }
        let req_body = req_body
            .into_inner()
            .context("failed to get inner vec from buffer")?;
        let resp = self
            .client
            .post(&self.endpoint)
            .send(&req_body)
            .context("failed to send ingest request")?;
        if resp.status() != StatusCode::OK && resp.status() != StatusCode::CONFLICT {
            bail!("server returned bad status code: {}", resp.status());
        }

        self.event_buffer.clear();

        Ok(())
    }
}

impl Drop for IngestClient {
    fn drop(&mut self) {
        if let Err(e) = self.ingest_batch() {
            tracing::error!(error = %e, "failed to ingest remaining events when IngestClient was dropped");
        }
    }
}
