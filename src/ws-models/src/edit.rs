use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Edit {
    #[serde(flatten)]
    pub shared: super::Shared,
    #[serde(flatten)]
    pub inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
    pub id: i64,
    pub notify_url: String,
    pub minor: bool,
    pub length: OldNew,
    pub revision: OldNew,
}

#[derive(Serialize, Deserialize)]
pub struct OldNew {
    old: u32,
    new: u32,
}
