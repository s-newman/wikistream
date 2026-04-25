use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "lowercase")]
pub enum Params {
    Create(UserId),
    ForceCreateLocal(UserId),
    Create2(UserId),
    ByeMail(UserId),
}

#[derive(Serialize, Deserialize)]
pub struct UserId {
    userid: u32,
}
