use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "snake_case")]
pub enum Params {
    Config {
        // TODO: parse as bool?
        #[serde(rename = "override")]
        _override: u8,
        autoreview: String,
        expiry: String,
        precedence: u8,
    },
    Reset {
        // TODO: parse as bool?
        #[serde(rename = "override")]
        _override: u8,
        autoreview: String,
        expiry: String,
        precedence: u8,
    },
    MoveStable {
        oldtitle: String,
    },
    Modify {
        // TODO: parse as bool?
        #[serde(rename = "override")]
        _override: u8,
        autoreview: String,
        expiry: String,
    },
}
