use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "kebab-case")]
pub enum Params {
    Merge {
        dest: String,
        mergepoint: String,
        mergerevid: String,
    },
    MergeInto {
        src: String,
        mergepoint: String,
        mergerevid: String,
    },
}
