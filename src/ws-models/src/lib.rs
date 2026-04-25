pub mod log;

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
pub struct FullEvent {
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

    #[serde(flatten)]
    inner: Type,
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Edit(Edit),
    Log(log::Log),
    New(New),
    Categorize,
}

#[derive(Serialize, Deserialize)]
pub struct Edit {
    id: u64,
    notify_url: String,
    minor: bool,
    length: OldNew,
    revision: OldNew,
}

#[derive(Serialize, Deserialize)]
pub struct OldNew {
    old: u32,
    new: u32,
}

#[derive(Serialize, Deserialize)]
pub struct JustNew {
    new: u32,
}

#[derive(Serialize, Deserialize)]
pub struct New {
    id: u64,
    patrolled: Option<bool>,
    length: JustNew,
    revision: JustNew,
}
