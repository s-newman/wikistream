use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "snake_case")]
pub enum Params {
    Delete(Vec<String>),
    Revision {
        #[serde(rename = "type")]
        _type: String,
        ids: Vec<u32>,
        ofield: u8,
        nfield: u8,
    },
    DeleteRedir(Vec<String>),
    Restore {
        count: Count,
    },
    Event {
        ids: Vec<u32>,
        ofield: u8,
        nfield: u8,
    },
    DeleteRedir2(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct Count {
    revisions: u8,
    files: u8,
}
