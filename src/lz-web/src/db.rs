//! Stuff for hooking up the DB to the lz web app.

use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};

/// An axum state object containing a connection pool to the SQLite DB.
///
/// This isn't all that useful (or safe) in a web request handler. Use
/// [`DbTransaction`] (extracted from the request) instead.
pub struct GlobalWebAppState {
    pool: lz_db::Connection,
    authentication_header_name: String,
    default_user_name: Option<String>,
}

impl GlobalWebAppState {
    pub fn new(
        pool: lz_db::Connection,
        authentication_header_name: String,
        default_user_name: Option<String>,
    ) -> Self {
        Self {
            pool,
            authentication_header_name,
            default_user_name,
        }
    }
}

/// A DB transaction that is started with each request.
///
/// The transaction does not get auto-committed at all: If your
/// request causes any changes to the DB, it _must_ call `commit`.
pub struct DbTransaction {
    txn: lz_db::Transaction,
}

impl Deref for DbTransaction {
    type Target = lz_db::Transaction;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}

impl DerefMut for DbTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.txn
    }
}

impl fmt::Debug for DbTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "txn({:?})", self.txn.user())
    }
}

pub struct DbTransactionRejection;
impl IntoResponse for DbTransactionRejection {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            "Could not authenticate your request",
        )
            .into_response()
    }
}

#[async_trait]
impl FromRequestParts<Arc<GlobalWebAppState>> for DbTransaction {
    type Rejection = DbTransactionRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<GlobalWebAppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .headers
            .get(&state.authentication_header_name)
            .map(|hv| hv.to_str())
            .or_else(|| {
                let username = state.default_user_name.as_ref().map(|s| Ok(s.as_str()));
                if let Some(default_username) = &username {
                    tracing::debug!(
                        ?default_username,
                        "request did not set user name, using default"
                    );
                }
                username
            });
        let txn = state
            .pool
            .begin_for_user(
                user.ok_or_else(|| {
                    tracing::error!("No user name could be determined from HTTP headers.");
                    DbTransactionRejection})?
                    .map_err(|e| {
                        tracing::warn!(error=%e, error_debug=?e, "HTTP headers contained a user name with invalid characters");
                        DbTransactionRejection
                    } )?,
            )
            .await
            .map_err(|e| {
                tracing::error!(error=%e, "failed to begin txn for user");
                DbTransactionRejection
            })?;
        Ok(DbTransaction { txn })
    }
}
