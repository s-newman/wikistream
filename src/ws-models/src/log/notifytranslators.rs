use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "camelCase")]
pub enum Params {
    Sent {
        languages_for_log: String,
        deadline_date: String,
        priority: String,
        sent_success: u8,
        sent_fail: u8,
        too_early: u8,
        #[serde(rename = "")]
        empty: String,
    },
}
