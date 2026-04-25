use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "kebab-case")]
pub enum Params {
    Approve(Vec<super::MultiType>),
    ApproveI(Vec<super::MultiType>),
    Unapprove(Vec<super::MultiType>),
}
