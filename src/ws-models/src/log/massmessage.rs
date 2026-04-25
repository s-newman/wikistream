use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Failure {
        subject: String,
        reason: String,
    },
    SkipNoUser {
        subject: String,
    },
    SkipOptOut {
        subject: String,
    },
    #[serde(rename_all = "camelCase")]
    Send {
        rev_id: u32,
        page_message: String,
    },
    SkipBadNs {
        subject: String,
    },
}
