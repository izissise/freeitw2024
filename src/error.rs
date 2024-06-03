use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Make our own error that wraps `anyhow::Error`.
pub struct HttpErr(anyhow::Error);

// Tell axum how to convert `HttpErr` into a response.
impl IntoResponse for HttpErr {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {}", self.0))
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, HttpErr>`. That way you don't need to do that manually.
impl<E> From<E> for HttpErr
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
