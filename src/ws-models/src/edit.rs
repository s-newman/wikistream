use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Edit {
    #[serde(flatten)]
    shared: super::Shared,
    #[serde(flatten)]
    inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
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
