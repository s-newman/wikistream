use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Hit {
        action: String,
        filter: String,
        actions: String,
        log: u32,
    },
}
