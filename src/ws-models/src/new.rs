use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct New {
    #[serde(flatten)]
    pub shared: super::Shared,
    #[serde(flatten)]
    pub inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
    pub id: i64,
    pub patrolled: Option<bool>,
    pub length: JustNew,
    pub revision: JustNew,
}

#[derive(Serialize, Deserialize)]
pub struct JustNew {
    new: u32,
}
