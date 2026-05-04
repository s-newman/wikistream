use crate::db;
use crate::db::DbError;
use crate::http::responses::{HttpResponse, HttpResult};
use crate::http::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::str::FromStr;
use ws_models::{Edit, Event, FullEvent};

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
    let mut events: Vec<Edit> = Vec::new();
    for line in body.lines() {
        let event = match Event::from_str(line) {
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
            continue;
        };
        // Only ingest edit events in english wiki for now
        let FullEvent::Edit(edit_event) = full_event else {
            continue;
        };
        if edit_event.shared.wiki != ENGLISH_WIKI {
            continue;
        }

        events.push(edit_event);
    }

    if events.is_empty() {
        return Ok(IngestResponse::Accepted.into());
    }

    match db::edit::bulk_create(&app_state.db_pool, events).await {
        Ok(_) => Ok(IngestResponse::Ok.into()),
        Err(DbError::Conflict) => Ok(IngestResponse::Conflict.into()),
        Err(e) => {
            tracing::error!(error = %e, "unexpected error from database");
            Ok(IngestResponse::Error {
                message: e.to_string(),
            }
            .into())
        }
    }
}
