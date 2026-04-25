use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Rights {
        #[serde(flatten)]
        groups: Groups,
        #[serde(flatten)]
        metadata: Metadata,
    },
    BlockAutoPromote {
        duration: u32,
        #[serde(flatten)]
        groups: Groups,
    },
    AutoPromote(Groups),
    RestoreAutoPromote {
        #[serde(flatten)]
        groups: Groups,
        #[serde(flatten)]
        metadata: Metadata,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Groups {
    #[serde(rename = "oldgroups")]
    old: Vec<String>,
    #[serde(rename = "newgroups")]
    new: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "oldmetadata")]
    old: Vec<super::Metadata>,
    #[serde(rename = "newmetadata")]
    new: Vec<super::Metadata>,
}
