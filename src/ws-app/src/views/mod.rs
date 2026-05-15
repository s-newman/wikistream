use anyhow::Context;
use chrono::NaiveDate;
use minijinja::{Environment, context, path_loader};
use serde::Serialize;

pub fn init() -> anyhow::Result<Environment<'static>> {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    Ok(env)
}

pub struct IndexArgs {
    pub previous_day: Option<NaiveDate>,
    pub date: NaiveDate,
    pub next_day: Option<NaiveDate>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub title: String,
    pub url: String,
    pub edits: i64,
    pub editors: i64,
    pub heat: u8,
}

pub fn index(env: &Environment<'static>, args: IndexArgs) -> anyhow::Result<String> {
    let tpl = env
        .get_template("index.html")
        .context("failed to get template")?;
    tpl.render(context!(
            previous_day => args.previous_day,
            date => args.date,
            next_day => args.next_day,
            pages => args.pages
    ))
    .context("failed to render template")
}
