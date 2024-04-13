//! Stuff for hooking up the DB to the lz web app.

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::ToStrError;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use lz_db::ReadWrite;

pub(crate) mod queries;

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

/// A read/write DB transaction that is started with each request.
///
/// The transaction does not get auto-committed at all: If your
/// request causes any changes to the DB, it _must_ call `commit`.
pub struct DbTransaction<M: lz_db::TransactionMode = lz_db::ReadOnly> {
    txn: lz_db::Transaction<M>,
}

impl DbTransaction<ReadWrite> {
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.txn.commit().await
    }
}

impl<M: lz_db::TransactionMode> Deref for DbTransaction<M> {
    type Target = lz_db::Transaction<M>;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}

impl<M: lz_db::TransactionMode> DerefMut for DbTransaction<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.txn
    }
}

impl<M: lz_db::TransactionMode> fmt::Debug for DbTransaction<M> {
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

fn user_name_from_headers<'a>(
    parts: &'a Parts,
    authentication_header_name: &str,
    default_user_name: Option<&'a str>,
) -> Option<Result<&'a str, ToStrError>> {
    parts
        .headers
        .get(authentication_header_name)
        .map(|hv| hv.to_str())
        .or_else(move || {
            if let Some(default_username) = default_user_name {
                tracing::debug!(
                    ?default_username,
                    "request did not set user name, using default"
                );
            }
            default_user_name.map(Ok)
        })
}

#[async_trait]
impl FromRequestParts<Arc<GlobalWebAppState>> for DbTransaction<lz_db::ReadOnly> {
    type Rejection = DbTransactionRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<GlobalWebAppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = user_name_from_headers(
            parts,
            &state.authentication_header_name,
            state.default_user_name.as_deref(),
        );
        let txn = state
            .pool
            .begin_ro_for_user(
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

#[async_trait]
impl FromRequestParts<Arc<GlobalWebAppState>> for DbTransaction<lz_db::ReadWrite> {
    type Rejection = DbTransactionRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<GlobalWebAppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = user_name_from_headers(
            parts,
            &state.authentication_header_name,
            state.default_user_name.as_deref(),
        );
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

#[cfg(test)]
mod test {
    use ::axum_test::TestServer;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::Router;
    use lz_db::{Bookmark, BookmarkId, IdType as _, NoId, UserId};
    use serde::Serialize;
    use testresult::TestResult;
    use url::Url;

    #[tokio::test]
    async fn deserialize_bookmark_types() -> TestResult {
        #[axum::debug_handler]
        async fn refer_to_existing_bookmark(
            axum::Form(bm): axum::Form<Bookmark<BookmarkId, UserId>>,
        ) -> &'static str {
            assert_eq!(bm.id.id(), 1);
            assert_eq!(bm.user_id.id(), 666);
            assert_eq!(bm.url.to_string(), "https://example.com/");
            "ok"
        }

        async fn refer_to_new_bookmark(
            axum::Form(bm): axum::Form<Bookmark<NoId, NoId>>,
        ) -> &'static str {
            assert_eq!(bm.id, NoId);
            assert_eq!(bm.user_id, NoId);
            assert_eq!(bm.url.to_string(), "https://example.com/");
            "ok"
        }

        let app = Router::new()
            .route("/existing", post(refer_to_existing_bookmark))
            .route("/new", post(refer_to_new_bookmark));
        let server = TestServer::new(app)?;
        #[derive(Serialize)]
        struct IncompleteBookmark {
            #[serde(skip_serializing_if = "Option::is_none")]
            id: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            user_id: Option<u64>,
            url: Url,
            title: &'static str,
            created_at: chrono::DateTime<chrono::Utc>,
        }
        let response = server
            .post("/existing")
            .form(&IncompleteBookmark {
                id: Some(1),
                user_id: Some(666),
                url: Url::parse("https://example.com")?,
                title: "example",
                created_at: Default::default(),
            })
            .await;
        assert_eq!(
            (response.text().as_str(), response.status_code()),
            ("ok", StatusCode::OK)
        );
        let response = server
            .post("/new")
            .form(&IncompleteBookmark {
                id: None,
                user_id: None,
                url: Url::parse("https://example.com")?,
                title: "example",
                created_at: Default::default(),
            })
            .await;
        assert_eq!(
            (response.text().as_str(), response.status_code()),
            ("ok", StatusCode::OK)
        );

        Ok(())
    }
}
