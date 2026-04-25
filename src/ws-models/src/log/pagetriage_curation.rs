use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "log_action", content = "log_params")]
#[serde(rename_all = "kebab-case")]
pub enum Params {
    ReviewedRedirect(Vec<String>),
    ReviewedArticle(Vec<String>),
    Tag(Tags),
    UnreviewedArticle(Vec<String>),
    Enqueue(Vec<String>),
    Delete(Tags),
    Insert(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct Tags {
    tags: Vec<String>,
}
