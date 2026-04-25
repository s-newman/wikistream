use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    RenameUser {
        olduser: String,
        newuser: String,
        edits: u32,
        derived: bool,
    },
}
