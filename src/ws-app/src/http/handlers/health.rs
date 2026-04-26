use crate::http::responses::HttpResponse;
use crate::{DbPool, db};
use axum::extract::State;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct HealthResponse {
    status: Health,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Health {
    Healthy,
    Unhealthy,
}

pub(super) async fn health(State(db_pool): State<DbPool>) -> HttpResponse<HealthResponse> {
    let mut status = Health::Healthy;

    if let Err(e) = db::ping(&db_pool).await {
        tracing::warn!(error = format!("{e:?}"), "database ping failed");
        status = Health::Unhealthy;
    }

    HttpResponse::ok(HealthResponse { status })
}
