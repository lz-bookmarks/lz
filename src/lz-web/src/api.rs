//! The lz-web API
//!
//! We use OpenAPI via the [utoipa] crate to generate an OpenAPI spec.

mod observability;
mod searching;

use std::sync::Arc;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use axum_valid::Valid;
use lz_db::{
    AssociatedLink, BookmarkId, BookmarkSearch, ExistingBookmark, ExistingTag, TagName, UserId,
};
use searching::TagQuery;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToResponse, ToSchema};
use validator::Validate;

use crate::db::{DbTransaction, GlobalWebAppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "Bookmarks", description = "Managing one's bookmarks")),
    paths(list_bookmarks, list_bookmarks_with_tag),
    security(),
    servers((url = "/api/v1/")),
    components(
        schemas(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, BookmarkId, ExistingBookmark, ExistingTag, Pagination, TagName, TagQuery),
        responses(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, ExistingBookmark, ExistingTag)
    )
)]
pub struct ApiDoc;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    let router = Router::new()
        .route("/bookmarks", get(list_bookmarks))
        .route("/bookmarks/tagged/*query", get(list_bookmarks_with_tag))
        .layer(CorsLayer::permissive());
    observability::add_layers(router)
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

/// Parameters that govern non-offset based pagination.
///
/// Pagination in `lz` works by getting the next page based on what
/// the previous page's last element was, aka "cursor-based
/// pagination". To that end, use the previous call's `nextCursor`
/// parameter into this call's `cursor` parameter.
#[derive(
    Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq, Hash, Validate, ToSchema,
)]
#[schema(default)]
#[serde(rename_all = "camelCase")]
struct Pagination {
    /// The last batch's last (oldest) bookmark ID
    #[schema(example = None)]
    cursor: Option<BookmarkId>,

    /// How many items to return
    #[schema(example = 50)]
    #[validate(range(min = 1, max = 500))]
    per_page: Option<u16>,
}

/// The response returned by the `list_bookmarks` API endpoint.
///
/// This response contains pagination information; if `next_cursor` is
/// set, passing that value to the `cursor` pagination parameter will
/// fetch the next page.
#[derive(Serialize, Debug, ToSchema, ToResponse)]
#[serde(rename_all = "camelCase")]
pub struct ListBookmarkResult {
    bookmarks: Vec<AnnotatedBookmark>,
    next_cursor: Option<BookmarkId>,
}

/// A bookmark, including tags and associations on it.
#[derive(Serialize, Debug, ToSchema, ToResponse)]
pub struct AnnotatedBookmark {
    bookmark: ExistingBookmark,
    tags: Vec<ExistingTag>,
    associations: Vec<AssociatedLink>,
}

/// List the user's bookmarks, newest to oldest.
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get,
    path = "/bookmarks",
    tag = "Bookmarks",
    params(
        ("cursor" = inline(Option<BookmarkId>),
            Query,
            style = Form,
        ),
        ("per_page" = inline(Option<u16>),
            Query,
            style = Form,
        ),
    ),
    responses(
        (status = 200, body = inline(ListBookmarkResult), description = "Lists all bookmarks"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn list_bookmarks(
    mut txn: DbTransaction,
    pagination: Option<Valid<Query<Pagination>>>,
) -> Result<Json<ListBookmarkResult>, (StatusCode, Json<ApiError>)> {
    let pagination = pagination.unwrap_or_default();
    let per_page = pagination.per_page.unwrap_or(20);
    let user_id = txn.user().id;
    let bms = txn
        .list_bookmarks_matching(
            vec![BookmarkSearch::User { id: user_id }],
            per_page,
            pagination.cursor,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;
    let mut taggings = txn.tags_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query tags for bookmarks");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let mut associations = txn.associated_links_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query for bookmark associations");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let mut next_cursor = None;
    let mut bookmarks = vec![];
    for (elt, bm) in bms.into_iter().enumerate() {
        if elt == usize::from(per_page) {
            // The "next cursor" element:
            next_cursor = Some(bm.id);
            break;
        }
        let id = bm.id;
        if let Some(tags) = taggings.remove(&id) {
            bookmarks.push(AnnotatedBookmark {
                bookmark: bm.clone(),
                tags,
                associations: associations.remove(&id).unwrap_or_else(std::vec::Vec::new),
            });
        } else {
            tracing::warn!(
                bookmark_id=?id,
                "somehow this bookmark seems to have appeared twice in the list of bookmarks?"
            );
        }
    }
    Ok(Json(ListBookmarkResult {
        bookmarks,
        next_cursor,
    }))
}

/// List bookmarks matching a tag, newest to oldest.
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get,
    path = "/bookmarks/tagged/{query}",
    tag = "Bookmarks",
    params(
        ("query" = inline(String), Path,),
        ("cursor" = inline(Option<BookmarkId>),
            Query,
            style = Form,
        ),
        ("per_page" = inline(Option<u16>),
            Query,
            style = Form,
        ),
    ),
    responses(
        (status = 200, body = inline(ListBookmarkResult), description = "Lists bookmarks matching the tag"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn list_bookmarks_with_tag(
    TagQuery { tags }: TagQuery,
    mut txn: DbTransaction,
    pagination: Option<Valid<Query<Pagination>>>,
) -> Result<Json<ListBookmarkResult>, (StatusCode, Json<ApiError>)> {
    let pagination = pagination.unwrap_or_default();
    let per_page = pagination.per_page.unwrap_or(20);
    let user_id = txn.user().id;
    let bms = txn
        .list_bookmarks_matching(
            tags.into_iter()
                .map(|name| BookmarkSearch::TagByName { name })
                .chain([BookmarkSearch::User { id: user_id }])
                .collect(),
            per_page,
            pagination.cursor,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;
    let mut taggings = txn.tags_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query tags for bookmarks");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let mut associations = txn.associated_links_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query for bookmark associations");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let mut next_cursor = None;
    let mut bookmarks = vec![];
    for (elt, bm) in bms.into_iter().enumerate() {
        if elt == usize::from(per_page) {
            // The "next cursor" element:
            next_cursor = Some(bm.id);
            break;
        }
        let id = bm.id;
        if let Some(tags) = taggings.remove(&id) {
            bookmarks.push(AnnotatedBookmark {
                bookmark: bm.clone(),
                tags,
                associations: associations.remove(&id).unwrap_or_else(std::vec::Vec::new),
            });
        } else {
            tracing::warn!(
                bookmark_id=?id,
                "somehow this bookmark seems to have appeared twice in the list of bookmarks?"
            );
        }
    }
    Ok(Json(ListBookmarkResult {
        bookmarks,
        next_cursor,
    }))
}
