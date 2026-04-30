use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::{IndexArgs, Page};
use crate::{db, views};
use anyhow::Context;
use axum::extract::{Query, State};
use axum::response::Html;
use chrono::{Days, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct IndexRequest {
    date: Option<NaiveDate>,
}

pub(super) async fn handler(
    State(app_state): State<AppState>,
    Query(request): Query<IndexRequest>,
) -> Result<Html<String>, HttpError> {
    let date = request.date.unwrap_or_else(|| Utc::now().date_naive());

    let pages: Vec<Page> = db::edit::most_edited_on_date(&app_state.db_pool, &date)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| Page {
            title: x.1.clone(),
            url: x.2.clone(),
            edits: x.0,
        })
        .collect();

    let tmpl_args = IndexArgs {
        date,
        pages,
        previous_day: date.checked_sub_days(Days::new(1)),
        next_day: date.checked_add_days(Days::new(1)),
    };
    let out = views::index(&app_state.env, tmpl_args).context("failed to render template")?;

    Ok(Html(out))
}
