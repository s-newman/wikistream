use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::Page;
use crate::{db, views};
use anyhow::Context;
use axum::extract::{Query, State};
use axum::response::Html;
use serde::Deserialize;
use sqlx::types::chrono::NaiveDate;

#[derive(Deserialize)]
pub(super) struct IndexRequest {
    date: NaiveDate,
}

pub(super) async fn handler(
    State(app_state): State<AppState>,
    Query(request): Query<IndexRequest>,
) -> Result<Html<String>, HttpError> {
    let pages = db::edit::most_edited_on_date(&app_state.db_pool, request.date)
        .await
        .context("failed to fetch from database")?;
    let out = views::index(
        &app_state.env,
        pages
            .iter()
            .map(|x| Page {
                title: x.1.clone(),
                url: x.2.clone(),
                edits: x.0,
            })
            .collect(),
    )
    .context("failed to render template")?;

    Ok(Html(out))
}
