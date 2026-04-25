use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Upload(Upload),
    Overwrite(Upload),
    Revert(Upload),
}

#[derive(Serialize, Deserialize)]
pub struct Upload {
    img_sha1: String,
    img_timestamp: String,
}
