use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Mark {
        revision: u32,
        changed: u16,
    },
    DeleteLok {
        target: String,
    },
    Associate {
        #[serde(rename = "aggregategroup")]
        aggregate_group: String,
        #[serde(rename = "aggregategroup-id")]
        aggregate_group_id: String,
    },
    PriorityLanguages {
        languages: String,
        force: bool,
        reason: String,
    },
    Discourage(Vec<String>),
    Unmark(Vec<String>),
}
