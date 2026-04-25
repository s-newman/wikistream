use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Log {
    #[serde(flatten)]
    shared: super::Shared,
    #[serde(flatten)]
    inner: Inner,
}

#[derive(Serialize, Deserialize)]
pub struct Inner {
    id: Option<u64>,
    log_id: u64,
    log_params: HashMap<String, Value>,
    log_action_comment: String,
}
