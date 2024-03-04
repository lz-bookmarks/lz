//! The lz-web API
//!
//! We use OpenAPI via the [utoipa] crate to generate an OpenAPI spec.

use std::sync::Arc;

use axum::{debug_handler, extract::Query, http::StatusCode, routing::get, Json, Router};
use axum_valid::Valid;
use lz_db::{Bookmark, BookmarkId, IdType as _, Tag, TagId, UserId};
use serde::{Deserialize, Serialize};
use static_assertions::assert_impl_all;
use thiserror::Error;
use utoipa::ToSchema;
use validator::Validate;

use crate::db::{DbTransaction, GlobalWebAppState};

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    Router::new().route("/bookmarks", get(list_bookmarks))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Error)]
enum ApiError {
    #[schema(example = "id = 1")]
    #[error("not found")]
    NotFound(String),

    #[schema()]
    #[error("error talking with the datastore")]
    #[serde(serialize_with = "serialize_db_error", skip_deserializing)]
    DatastoreError(#[from] sqlx::Error),
}

fn serialize_db_error<S>(_err: &sqlx::Error, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str("(nasty DB error omitted)")
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, Validate)]
struct Pagination {
    last_seen: Option<BookmarkId>,
    #[validate(range(min = 1, max = 500))]
    per_page: Option<u16>,
}

assert_impl_all!(Pagination: Validate);

#[derive(Serialize, Debug, ToSchema)]
struct AnnotatedBookmark {
    bookmark: Bookmark<BookmarkId, UserId>,
    tags: Vec<Tag<TagId>>,
}

/// Lists the user's bookmarks, newest to oldest
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get, path = "/api/bookmarks",
    responses(
        (status = 200, description = "Lists all bookmarks"),
    ),
)]
#[tracing::instrument(skip(txn))]
async fn list_bookmarks(
    mut txn: DbTransaction,
    Valid(Query(pagination)): Valid<Query<Pagination>>,
) -> Result<Json<Vec<AnnotatedBookmark>>, (StatusCode, Json<ApiError>)> {
    let bms = txn
        .list_bookmarks(pagination.per_page.unwrap_or(20), pagination.last_seen)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;
    let mut taggings = txn.tags_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query tags for bookmarks");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let mut bookmarks = vec![];
    for bm in bms {
        let id = bm.id;
        bookmarks.push(AnnotatedBookmark {
            bookmark: bm,
            tags: taggings.remove(&id).ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::NotFound(format!(
                        "Bookmark ID {} was mistreated",
                        id.id()
                    ))),
                )
            })?,
        })
    }
    Ok(Json(bookmarks))
}
