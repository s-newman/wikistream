use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "snake_case")]
pub enum Params {
    Move(Move),
    MoveRedir(Move),
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    target: String,
    // TODO: parse as bool? i think values are always "0" or "1"
    noredir: String,
}
