use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "kebab-case")]
pub enum Params {
    WishStatusChange { old: String, new: String },
    WishCreate(Vec<String>),
}
