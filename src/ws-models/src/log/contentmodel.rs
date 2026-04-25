use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    New { oldmodel: bool, newmodel: String },
    Change { oldmodel: String, newmodel: String },
}
