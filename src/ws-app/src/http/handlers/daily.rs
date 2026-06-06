use anyhow::Context;
use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::ACCEPT;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Days, NaiveDate, NaiveTime, Utc};
use minijinja::Environment;
use serde::{Deserialize, Serialize};

use crate::http::responses::HttpError;
use crate::http::server::AppState;
use crate::views::{DailyArgs, Page};
use crate::{db, views};

const MIDNIGHT_TIME: NaiveTime =
    NaiveTime::from_hms_opt(0, 0, 0).expect("bad MIDNIGHT_TIME definition");

const TEXT_HTML: HeaderValue = HeaderValue::from_static("text/html");
const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");
const MIME_ANY: HeaderValue = HeaderValue::from_static("*/*");

enum ResponseType {
    Html,
    Json,
    NotAcceptable,
}

#[derive(Serialize, Deserialize)]
pub struct DailyTopPagesOutput {
    day: NaiveDate,
    pages: Vec<Page>,
}

impl DailyTopPagesOutput {
    pub(super) fn new(day: NaiveDate, pages: Vec<Page>) -> Self {
        Self { day, pages }
    }

    pub(super) fn into_response(
        self,
        headers: &HeaderMap,
        env: &Environment,
    ) -> Result<Response, HttpError> {
        match get_response_type(headers) {
            ResponseType::Html => {
                let tmpl_args = DailyArgs {
                    date: self.day,
                    pages: self.pages,
                    previous_day: self.day.checked_sub_days(Days::new(1)),
                    next_day: self.day.checked_add_days(Days::new(1)),
                };
                let out = views::daily(env, tmpl_args).context("failed to render template")?;
                Ok(Html(out).into_response())
            }
            ResponseType::Json => Ok(Json(self).into_response()),
            ResponseType::NotAcceptable => Ok(Response::builder()
                .status(StatusCode::NOT_ACCEPTABLE)
                .body(Body::from("Not Acceptable"))
                .context("failed to construct Not Acceptable response")?),
        }
    }

    pub fn day(&self) -> &NaiveDate {
        &self.day
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }
}

fn get_response_type(headers: &HeaderMap) -> ResponseType {
    match headers.get(ACCEPT) {
        Some(x) => {
            if *x == APPLICATION_JSON {
                ResponseType::Json
            } else if *x == TEXT_HTML || *x == MIME_ANY {
                ResponseType::Html
            } else {
                ResponseType::NotAcceptable
            }
        }
        None => ResponseType::Html,
    }
}

pub(super) async fn date(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Path(date): Path<NaiveDate>,
) -> Result<Response, HttpError> {
    let pages: Vec<Page> = db::edit::most_edited_on_date(&app_state.db_pool, &date)
        .await
        .context("failed to fetch from database")?
        .iter()
        .map(|x| x.into())
        .collect();

    DailyTopPagesOutput::new(date, pages).into_response(&headers, &app_state.env)
}

#[derive(Deserialize)]
pub(super) struct IndexRequest {
    date: Option<NaiveDate>,
}

pub(super) async fn index(
    headers: HeaderMap,
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

    DailyTopPagesOutput::new(end.date_naive(), pages).into_response(&headers, &app_state.env)
}
