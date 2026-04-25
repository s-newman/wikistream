use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    #[serde(rename_all = "kebab-case")]
    Group {
        language: String,
        group_label: String,
        old_state: String,
        new_state: String,
    },
    Message {
        revision: u32,
    },
    Unfuzzy(Vec<String>),
}
