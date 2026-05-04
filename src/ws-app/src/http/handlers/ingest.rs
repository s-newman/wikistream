use crate::db::DbError;
use crate::http::responses::{HttpResponse, HttpResult};
use crate::http::server::AppState;
use crate::{DbPool, db};
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
    #[allow(dead_code)]
    Accepted,
    #[allow(dead_code)]
    Conflict,
    Error {
        message: String,
    },
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

        ingest_one(&app_state.db_pool, event)
            .await
            .context("unexpected error from database")?;
    }

    Ok(IngestResponse::Ok.into())
}

async fn ingest_one(db_pool: &DbPool, event: Event) -> Result<(), DbError> {
    // Skip canary events
    let Event::Event(full_event) = event else {
        return Ok(());
    };

    let FullEvent::Edit(edit_event) = full_event else {
        return Ok(());
    };
    if edit_event.shared.wiki != ENGLISH_WIKI {
        return Ok(());
    }

    match db::edit::create(db_pool, edit_event).await {
        Ok(_) => Ok(()),
        Err(DbError::Conflict) => Ok(()),
        Err(e) => Err(e),
    }
}
