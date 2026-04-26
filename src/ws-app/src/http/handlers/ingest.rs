use crate::http::responses::{HttpResponse, HttpResult};
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
    Error { message: String },
}

impl From<IngestResponse> for HttpResponse<IngestResponse> {
    fn from(value: IngestResponse) -> Self {
        match &value {
            IngestResponse::Ok => Self::new(StatusCode::OK, value),
            IngestResponse::Error { message: _ } => Self::new(StatusCode::BAD_REQUEST, value),
        }
    }
}

pub(super) async fn ingest(
    State(db_pool): State<DbPool>,
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

    match full_event {
        FullEvent::Categorize(x) => db::categorize::create(&db_pool, x).await,
        FullEvent::Edit(x) => db::edit::create(&db_pool, x).await,
        FullEvent::Log(x) => db::log::create(&db_pool, x).await,
        FullEvent::New(x) => db::new::create(&db_pool, x).await,
    }
    .context("failed to insert event into database")?;

    Ok(IngestResponse::Ok.into())
}
