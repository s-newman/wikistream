use anyhow::Context;
use minijinja::{Environment, context};
use serde::Serialize;

pub fn init() -> anyhow::Result<Environment<'static>> {
    let mut env = Environment::new();

    env.add_template("index.html", include_str!("templates/index.html"))
        .context("failed to load template")?;

    Ok(env)
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub title: String,
    pub url: String,
    pub edits: i64,
}

pub fn index(env: &Environment<'static>, p: Vec<Page>) -> anyhow::Result<String> {
    let tpl = env
        .get_template("index.html")
        .context("failed to get template")?;
    tpl.render(context!( pages => p ))
        .context("failed to render template")
}
