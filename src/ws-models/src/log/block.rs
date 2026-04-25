use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Reblock {
        duration: String,
        flags: String,
        sitewide: bool,
        #[serde(rename = "blockId")]
        block_id: u32,
    },
    Block {
        duration: String,
        flags: String,
        sitewide: bool,
        #[serde(rename = "blockId")]
        block_id: u32,
    },
    Unblock {
        #[serde(rename = "blockId")]
        block_id: u32,
    },
}
