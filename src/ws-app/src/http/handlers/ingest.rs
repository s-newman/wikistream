use crate::db;
use crate::db::DbError;
use crate::http::responses::{HttpResponse, HttpResult};
use crate::http::server::AppState;
use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::str::FromStr;
use ws_models::{Event, FullEvent};

#[derive(Serialize)]
#[serde(tag = "status")]
#[serde(rename_all = "lowercase")]
pub(super) enum IngestResponse {
    Ok,
    Accepted,
    Conflict,
    Error { message: String },
}

impl From<IngestResponse> for HttpResponse<IngestResponse> {
    fn from(value: IngestResponse) -> Self {
        match &value {
            IngestResponse::Ok => Self::new(StatusCode::OK, value),
            IngestResponse::Accepted => Self::new(StatusCode::ACCEPTED, value),
            IngestResponse::Conflict => Self::new(StatusCode::CONFLICT, value),
            IngestResponse::Error { message: _ } => Self::new(StatusCode::BAD_REQUEST, value),
        }
    }
}

const ENGLISH_WIKI: &str = "enwiki";

pub(super) async fn ingest(
    State(app_state): State<AppState>,
    body: String,
) -> HttpResult<IngestResponse> {
    let event = match Event::from_str(&body) {
        Ok(x) => x,
        Err(e) => {
            return Ok(IngestResponse::Error {
                message: e.to_string(),
            }
            .into());
        }
    };

    // Skip canary events
    let Event::Event(full_event) = event else {
        return Ok(IngestResponse::Ok.into());
    };

    let FullEvent::Edit(edit_event) = full_event else {
        return Ok(IngestResponse::Accepted.into());
    };
    if edit_event.shared.wiki != ENGLISH_WIKI {
        return Ok(IngestResponse::Accepted.into());
    }
    let result = db::edit::create(&app_state.db_pool, edit_event).await;

    Ok(match result {
        Ok(_) => IngestResponse::Ok.into(),
        Err(DbError::Conflict) => IngestResponse::Conflict.into(),
        Err(e) => Err(e).context("unexpected error from database")?,
    })
}
