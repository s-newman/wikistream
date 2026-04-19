mod event;

pub use event::Event;
use std::error::Error;
use std::io::{ErrorKind, Read};

use crate::event::EventReader;
use ureq::Body;
use ureq::http::Response;

pub struct EventSource {
    url: String,
    response: Option<Response<Body>>,
    event_reader: EventReader,
    buf: Vec<u8>,
}

const BUFFER_SZ: usize = 1024 * 100;

impl EventSource {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        Self {
            url: url.as_ref().into(),
            response: None,
            event_reader: EventReader::default(),
            buf: vec![0u8; BUFFER_SZ],
        }
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        tracing::info!(
            event_id = self.event_reader.last_event_id(),
            "connecting to stream"
        );

        let mut req = ureq::get(&self.url).header(
            "User-Agent",
            "wikistream/dev (nwmn_devcontact@fastmail.com) ureq/??",
        );

        if !self.event_reader.last_event_id().is_empty() {
            req = req.header("Last-Event-ID", self.event_reader.last_event_id());
        }

        self.response = match req.call() {
            Ok(resp) => Some(resp),
            Err(e) => {
                tracing::error!(error = %e, "unexpected error connecting to stream");
                return Err(Box::new(e));
            }
        };

        Ok(())
    }
}

impl Iterator for EventSource {
    type Item = Result<Event, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Attempt to parse from what we've read so far
            if let Some(event) = self.event_reader.read_event() {
                return Some(Ok(event));
            }

            // Read another chunk
            let resp = match &mut self.response {
                Some(resp) => resp,
                None => {
                    if let Err(e) = self.connect() {
                        return Some(Err(e));
                    }
                    self.response
                        .as_mut()
                        .expect("response was none immediately after connection")
                }
            };
            let n = match resp.body_mut().as_reader().read(&mut self.buf) {
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    // Need to reconnect, clear response to reconnect on next loop
                    self.event_reader.reset();
                    self.response = None;
                    continue;
                }
                Err(e) => {
                    // TODO: retry with backoff X number of times before exiting
                    tracing::error!(error = %e, "unexpected error reading from socket");
                    return Some(Err(Box::new(e)));
                }
            };

            self.event_reader.add(&self.buf[..n]);
        }
    }
}
