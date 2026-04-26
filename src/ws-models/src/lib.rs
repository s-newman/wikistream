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
use std::str::FromStr;

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

impl FromStr for Event {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
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
    pub schema: String,
    pub meta: Meta,
    pub namespace: i16,
    pub title: String,
    pub title_url: String,
    pub comment: String,
    pub timestamp: i64,
    pub user: String,
    pub bot: bool,
    pub server_url: String,
    pub server_name: String,
    pub server_script_path: String,
    pub wiki: String,
    pub parsedcomment: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub uri: String,
    pub request_id: String,
    pub id: String,
    pub domain: String,
    pub stream: String,
    pub dt: DateTime<Utc>,
    pub topic: String,
    pub partition: i16,
    pub offset: i64,
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
