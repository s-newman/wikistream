use crate::db;
use crate::http::handlers::daily::DailyTopPagesOutput;
use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::Page;
use anyhow::Context;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::Utc;

pub(super) async fn handler(
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<Response, HttpError> {
    let date = Utc::now().date_naive();

    let pages: Vec<Page> = db::edit::most_edited_on_date(&app_state.db_pool, &date)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| x.into())
        .collect();

    DailyTopPagesOutput::new(date, pages).into_response(&headers, &app_state.env)
}
