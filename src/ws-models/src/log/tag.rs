use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    #[serde(rename_all = "camelCase")]
    Update {
        revid: String,
        logid: u32,
        tags_added: Vec<String>,
        tags_added_count: u32,
        tags_removed: Vec<String>,
        tags_removed_count: u32,
        initial_tags: Vec<String>,
    },
}
