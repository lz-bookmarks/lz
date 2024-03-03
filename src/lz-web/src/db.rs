//! Stuff for hooking up the DB to the lz web app.

use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Extension,
};

/// An axum state object containing a connection pool to the SQLite DB.
///
/// This isn't all that useful (or safe) in a web request handler. Use
/// [`DbTransaction`] (extracted from the request) instead.
pub struct GlobalWebAppState {
    pub pool: lz_db::Connection,
    pub authentication_header_name: String,
    pub default_user_name: Option<String>,
}

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
            .or_else(|| state.default_user_name.as_ref().map(|s| Ok(s.as_str())));
        let txn = state
            .pool
            .begin_for_user(
                user.ok_or(DbTransactionRejection)?
                    .map_err(|_| DbTransactionRejection)?,
            )
            .await
            .map_err(|e| {
                tracing::error!(error=%e, "failed to begin txn for user");
                DbTransactionRejection
            })?;
        Ok(DbTransaction { txn })
    }
}
