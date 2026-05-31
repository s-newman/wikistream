use anyhow::Context;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Days, NaiveDate, NaiveTime, Utc};
use serde::Deserialize;

use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::{DailyArgs, Page};
use crate::{db, views};

const MIDNIGHT_TIME: NaiveTime =
    NaiveTime::from_hms_opt(0, 0, 0).expect("bad MIDNIGHT_TIME definition");

pub(super) async fn date(
    State(app_state): State<AppState>,
    Path(date): Path<NaiveDate>,
) -> Result<Html<String>, HttpError> {
    let end = date.and_time(MIDNIGHT_TIME).and_utc();
    let start = end - Days::new(1);

    let pages: Vec<Page> = db::edit::most_edited_between(&app_state.db_pool, &start, &end)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| x.into())
        .collect();

    let tmpl_args = DailyArgs {
        date: end.date_naive(),
        pages,
        previous_day: end.checked_sub_days(Days::new(1)).map(|e| e.date_naive()),
        next_day: end.checked_add_days(Days::new(1)).map(|nd| nd.date_naive()),
    };
    let out = views::daily(&app_state.env, tmpl_args).context("failed to render template")?;

    Ok(Html(out))
}

#[derive(Deserialize)]
pub(super) struct IndexRequest {
    date: Option<NaiveDate>,
}

pub(super) async fn index(
    State(app_state): State<AppState>,
    Query(request): Query<IndexRequest>,
) -> Result<Response, HttpError> {
    if let Some(date) = request.date {
        return Ok(Redirect::to(&format!("/daily/{date}")).into_response());
    }
    let end = request
        .date
        .map(|d| d.and_time(MIDNIGHT_TIME).and_utc())
        .unwrap_or_else(Utc::now);
    let start = end - Days::new(1);

    let pages: Vec<Page> = db::edit::most_edited_between(&app_state.db_pool, &start, &end)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| x.into())
        .collect();

    let tmpl_args = DailyArgs {
        date: end.date_naive(),
        pages,
        previous_day: end.checked_sub_days(Days::new(1)).map(|e| e.date_naive()),
        next_day: end.checked_add_days(Days::new(1)).map(|nd| nd.date_naive()),
    };
    let out = views::daily(&app_state.env, tmpl_args).context("failed to render template")?;

    Ok(Html(out).into_response())
}
