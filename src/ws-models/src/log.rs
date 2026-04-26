use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Log {
    #[serde(flatten)]
    pub shared: super::Shared,
    #[serde(flatten)]
    pub inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
    pub id: Option<i64>,
    pub log_id: i64,
    pub log_params: HashMap<String, Value>,
    pub log_action_comment: String,
}
