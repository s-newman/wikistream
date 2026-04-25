use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Categorize {
    #[serde(flatten)]
    shared: super::Shared,
}
