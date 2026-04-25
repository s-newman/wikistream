use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "snake_case")]
pub enum Params {
    Protect {
        description: String,
        cascade: bool,
        details: Vec<Detail>,
    },
    Modify {
        description: String,
        cascade: bool,
        details: Vec<Detail>,
    },
    MoveProt {
        oldtitle: String,
    },
    Unprotect(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct Detail {
    #[serde(rename = "type")]
    _type: String,
    level: String,
    expiry: String,
}
