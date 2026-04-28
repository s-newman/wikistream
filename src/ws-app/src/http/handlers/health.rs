use crate::db;
use crate::http::responses::HttpResponse;
use crate::http::server::AppState;
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

pub(super) async fn health(State(app_state): State<AppState>) -> HttpResponse<HealthResponse> {
    let mut status = Health::Healthy;

    if let Err(e) = db::ping(&app_state.db_pool).await {
        tracing::warn!(error = format!("{e:?}"), "database ping failed");
        status = Health::Unhealthy;
    }

    HttpResponse::ok(HealthResponse { status })
}
