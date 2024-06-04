use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Make our own error that wraps `anyhow::Error`.
#[derive(Error, Debug)]
pub enum HttpErr {
    /// a generic error
    #[error("generic")]
    Err(#[from] anyhow::Error),
    /// An http status
    #[error("status code")]
    Status(StatusCode),
}

// Tell axum how to convert `HttpErr` into a response.
impl IntoResponse for HttpErr {
    fn into_response(self) -> Response {
        match self {
            Self::Err(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {e}"))
                    .into_response()
            }
            Self::Status(sc) => sc.into_response(),
        }
    }
}

// This enables using `?` on functions that return `Result<_, StatusCode>` to turn them into
// `Result<_, HttpErr>`. That way you don't need to do that manually.
impl From<StatusCode> for HttpErr {
    fn from(sc: StatusCode) -> Self {
        Self::Status(sc)
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, HttpErr>`. That way you don't need to do that manually.
// impl<E> From<E> for HttpErr
// where
//     E: Into<anyhow::Error>,
// {
//     fn from(err: E) -> Self {
//         Self::Err(err.into())
//     }
// }
