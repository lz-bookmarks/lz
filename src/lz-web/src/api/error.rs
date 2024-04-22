use std::fmt;

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::LookupError;

/// An error that can occur in the API.
///
/// This type is [`IntoResponse`].
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub(crate) enum ApiError {
    #[schema()]
    NotFound,

    #[schema()]
    #[serde(serialize_with = "serialize_db_error", skip_deserializing)]
    DatastoreError(sqlx::Error),

    #[schema()]
    #[serde(serialize_with = "serialize_lookup_error", skip_deserializing)]
    Lookup(LookupError),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "not found"),
            ApiError::DatastoreError(_) => write!(f, "datastore error"),
            ApiError::Lookup(e) => write!(f, "HTTP error {e}"),
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse<'a> {
            error_message: &'a str,
        }

        let error_as_text = self.to_string();
        let (status, error_message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, &error_as_text),
            ApiError::DatastoreError(inner) => {
                tracing::error!(error=%inner, error_debug=?inner, "datastore error");
                (StatusCode::INTERNAL_SERVER_ERROR, &error_as_text)
            }
            ApiError::Lookup(error) => {
                tracing::warn!(error_debug=?error, %error, "HTTP error");
                (StatusCode::BAD_REQUEST, &error_as_text)
            }
        };
        (status, Json(ErrorResponse { error_message })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(other: sqlx::Error) -> Self {
        ApiError::DatastoreError(other)
    }
}
impl From<LookupError> for ApiError {
    fn from(other: LookupError) -> Self {
        ApiError::Lookup(other)
    }
}

fn serialize_db_error<S>(_err: &sqlx::Error, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str("(nasty DB error omitted)")
}

fn serialize_lookup_error<S>(err: &LookupError, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("lookup error: {}", err))
}
