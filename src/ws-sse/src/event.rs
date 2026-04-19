//! SSE event stream parser.
//!
//! Ref: https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream

use std::collections::VecDeque;

pub struct Event {
    pub typ: String,
    pub id: String,
    pub data: String,
    pub retry: Option<u32>,
}

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;

pub struct EventReader {
    buf: VecDeque<u8>,
    latest_event_id: String,
    last_event_id: String,
    event_type: String,
    data: String,
    retry: Option<u32>,
}

impl Default for EventReader {
    fn default() -> Self {
        Self {
            buf: VecDeque::new(),
            latest_event_id: String::new(),
            last_event_id: String::new(),
            event_type: String::from("message"),
            data: String::new(),
            retry: None,
        }
    }
}

impl EventReader {
    /// Clear unparsed line buffer and any partially-parsed event data.
    ///
    /// To be called when reconnecting to the event stream after a disconnect, since the server is
    /// going to resume with a full event (not the remainder of whatever partial event we were in
    /// the middle of).
    pub fn reset(&mut self) {
        self.buf.clear();
        self.event_type.clear();
        self.event_type.push_str("message");
        self.data.clear();
        self.retry = None;
    }

    pub fn add(&mut self, buf: &[u8]) {
        self.buf.extend(buf);
    }

    pub fn last_event_id(&self) -> &str {
        &self.last_event_id
    }

    pub fn read_event(&mut self) -> Option<Event> {
        // ref: https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation
        while let Some(line) = self.read_line() {
            // If the line is empty, dispatch the event
            if line.is_empty() {
                let event = Event {
                    typ: self.event_type.clone(),
                    id: self.latest_event_id.clone(),
                    data: self.data.clone(),
                    retry: self.retry,
                };

                // Only set the last_event_id after parsing a full event. If we only parse part of
                // this event, we want to resume at that event so we don't miss anything.
                self.last_event_id = self.latest_event_id.clone();

                // reset buffers
                self.event_type.clear();
                self.event_type.push_str("message");
                self.data.clear();
                self.retry = None;

                return Some(event);
            }

            // If the line starts with a colon character, ignore the line
            if line.starts_with(':') {
                continue;
            }

            // If the line contains a colon character, characters before the colon are the field
            // name and characters after (minus a leading space, if present) are the value
            if let Some((name, raw_val)) = line.split_once(':') {
                // Trim single leading space if present
                let value = raw_val.strip_prefix(' ').unwrap_or(raw_val);

                match name {
                    "event" => self.event_type = value.to_string(),
                    "data" => self.data.push_str(value),
                    "id" => self.latest_event_id = value.to_string(),
                    "retry" => {
                        // Field is ignored if it can't be parsed as an integer
                        if let Ok(v) = value.parse::<u32>() {
                            self.retry = Some(v);
                        }
                    }
                    // "otherwise, the field is ignored"
                    _ => continue,
                }
            }
        }

        // We weren't able to parse a full event
        None
    }

    fn read_line(&mut self) -> Option<String> {
        let mut line: Vec<u8> = Vec::new();

        while let Some(byte) = self.buf.pop_front_if(|b| *b != CR && *b != LF) {
            line.push(byte);
        }

        // If buffer is empty, we hit EOF rather than end of line. Add the partial line back to the
        // buffer to be reused later.
        if self.buf.is_empty() {
            self.buf.append(&mut line.into());
            return None;
        }

        // Skip over end-of-line, which could be CRLF, LF, or just CR
        match self.buf.pop_front() {
            Some(CR) => {
                self.buf.pop_front();
                self.buf.pop_front_if(|b| *b == LF);
            }
            Some(LF) => (),
            // If the buffer is empty, we hit EOF rather than end of line. Add the partial line back
            // to the buffer to be reused later.
            None => {
                self.buf.append(&mut line.into());
                return None;
            }
            // Our `while let` loop will only ever stop at a CR or LF character, so we should never
            // hit this branch.
            Some(_) => unreachable!(),
        }

        // TODO: don't panic
        Some(line.try_into().expect("invalid UTF-8 line"))
    }
}
