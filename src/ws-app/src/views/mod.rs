use anyhow::Context;
use chrono::NaiveDate;
use minijinja::{Environment, context, path_loader};
use serde::Serialize;

use crate::db::edit::EditPageView;

pub fn init() -> anyhow::Result<Environment<'static>> {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    Ok(env)
}

pub struct DailyArgs {
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

impl Page {
    fn get_heat(edits: i64, editors: i64) -> u8 {
        match (editors * 100) / edits {
            ..=20 => 0,
            21..=36 => 1,
            37..=52 => 2,
            53..=68 => 3,
            69..=84 => 4,
            85.. => 5,
        }
    }
}

impl From<&EditPageView> for Page {
    fn from(value: &EditPageView) -> Self {
        Self {
            title: value.title.clone(),
            url: value.title_url.clone(),
            edits: value.total,
            editors: value.editors,
            heat: Self::get_heat(value.total, value.editors),
        }
    }
}

impl From<EditPageView> for Page {
    fn from(value: EditPageView) -> Self {
        Self {
            title: value.title,
            url: value.title_url,
            edits: value.total,
            editors: value.editors,
            heat: Self::get_heat(value.total, value.editors),
        }
    }
}

pub fn daily(env: &Environment<'static>, args: DailyArgs) -> anyhow::Result<String> {
    let tpl = env
        .get_template("daily.html")
        .context("failed to get template")?;
    tpl.render(context!(
            previous_day => args.previous_day,
            date => args.date,
            next_day => args.next_day,
            pages => args.pages
    ))
    .context("failed to render template")
}
