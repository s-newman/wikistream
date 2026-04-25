pub mod categorize;
pub use categorize::Categorize;
pub mod edit;
pub use edit::Edit;
pub mod log;
pub use log::Log;
pub mod new;
pub use new::New;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Event(FullEvent),
    Canary {
        #[serde(rename = "$schema")]
        schema: String,
        meta: CanaryMeta,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum FullEvent {
    Categorize(Categorize),
    Edit(Edit),
    Log(Log),
    New(New),
}

#[derive(Serialize, Deserialize)]
pub struct Shared {
    #[serde(rename = "$schema")]
    schema: String,
    meta: Meta,
    namespace: i16,
    title: String,
    title_url: String,
    comment: String,
    timestamp: u64,
    user: String,
    bot: bool,
    server_url: String,
    server_name: String,
    server_script_path: String,
    wiki: String,
    parsedcomment: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    uri: String,
    request_id: String,
    id: String,
    domain: String,
    stream: String,
    dt: DateTime<Utc>,
    topic: String,
    partition: u16,
    offset: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CanaryMeta {
    request_id: String,
    id: String,
    domain: String,
    stream: String,
    dt: DateTime<Utc>,
    topic: String,
    partition: u16,
    offset: u64,
}
