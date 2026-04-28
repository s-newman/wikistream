use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Error type for failable handlers to use.
///
/// Wraps [HttpError] so unknown server-side errors can be easily handled with
/// the question mark operator.
pub type HttpResult<T> = Result<HttpResponse<T>, HttpError>;

/// Wrapper type for returning any HTTP response with a specific status code.
/// Prefer using this over directly returning another type that implements
/// [axum::response::IntoResponse] on the "okay" path, because wrapping a
/// response body in this struct makes the status code explicit.
pub struct HttpResponse<T: Serialize> {
    status_code: StatusCode,
    body: T,
}

impl<T: Serialize> HttpResponse<T> {
    pub fn new(status_code: StatusCode, body: T) -> Self {
        Self { status_code, body }
    }

    pub fn ok(body: T) -> Self {
        Self::new(StatusCode::OK, body)
    }
}

impl<T: Serialize> IntoResponse for HttpResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

/// Represents any errors that could be returned by handler functions.
#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    /// `500 Internal Server Error` for [anyhow::Error].
    #[error("an internal server error occurred")]
    Unknown(#[from] anyhow::Error),
}

impl HttpError {
    /// Used by [axum::response::IntoResponse] to map error variants to status
    /// codes.
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown(e) => {
                // anyhow's errors get erased so we don't leak error details to
                // end users. We still want to record that error information
                // somewhere though, so we'll log the error while converting it
                // since this code path will be called automatically for us
                // after a handler function returns.
                tracing::error!(error = ?e, "unknown error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
