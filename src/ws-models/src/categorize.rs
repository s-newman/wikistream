use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Categorize {
    #[serde(flatten)]
    pub shared: super::Shared,
}
