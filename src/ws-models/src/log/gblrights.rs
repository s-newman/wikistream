use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    #[serde(rename_all = "camelCase")]
    UserGroups {
        old_groups: Vec<String>,
        new_groups: Vec<String>,
        old_metadata: Vec<super::Metadata>,
        new_metadata: Vec<super::Metadata>,
    },
}
