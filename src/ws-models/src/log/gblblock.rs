use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    GBlock(Block),
    GUnblock(Vec<String>),
    Modify(Block),
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    expiry: String,
    flags: Vec<String>,
}
