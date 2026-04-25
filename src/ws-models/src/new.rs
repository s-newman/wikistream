use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct New {
    #[serde(flatten)]
    shared: super::Shared,
    #[serde(flatten)]
    inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
    id: u64,
    patrolled: Option<bool>,
    length: JustNew,
    revision: JustNew,
}

#[derive(Serialize, Deserialize)]
pub struct JustNew {
    new: u32,
}
