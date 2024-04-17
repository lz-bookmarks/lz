//! The lz-web API
//!
//! We use OpenAPI via the [utoipa] crate to generate an OpenAPI spec.

mod observability;
mod searching;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::routing::post;
use axum::{debug_handler, Json, Router};
use lz_db::{
    AssociatedLink, BookmarkId, BookmarkSearch, BookmarkSearchDateParams,
    BookmarkSearchDatetimeField, BookmarkSearchDatetimeOrientation, DateInput, ExistingBookmark,
    ExistingTag, NewBookmark, NoId, ReadWrite, TagId, TagName, UserId,
};
use searching::TagQuery;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower_http::cors::CorsLayer;
use utoipa::{IntoParams, OpenApi, ToResponse, ToSchema};

use crate::db::queries::{list_bookmarks, AnnotatedBookmark, ListResult, Pagination};
use crate::db::{DbTransaction, GlobalWebAppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "Bookmarks", description = "Managing one's bookmarks")),
    paths(list_bookmarks_matching, create_bookmark),
    security(),
    servers((url = "/api/v1/")),
    components(
        schemas(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, BookmarkId, ExistingBookmark, ExistingTag, Pagination, TagName, TagQuery, ListRequest, BookmarkSearch, BookmarkSearchDateParams, DateInput, BookmarkSearchDatetimeField, BookmarkSearchDatetimeOrientation, TagId, NoId, BookmarkCreateRequest),
        responses(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, ExistingBookmark, ExistingTag)
    )
)]
pub struct ApiDoc;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    let router = Router::new()
        .route("/bookmarks", post(list_bookmarks_matching))
        .route("/bookmark/create", post(create_bookmark))
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

/// The response returned by the `list_bookmarks` API endpoint.
///
/// This response contains pagination information; if `next_cursor` is
/// set, passing that value to the `cursor` pagination parameter will
/// fetch the next page.
#[derive(Serialize, Debug, ToSchema, ToResponse)]
pub struct ListBookmarkResult {
    bookmarks: Vec<AnnotatedBookmark>,
    next_cursor: Option<BookmarkId>,
}

/// A bookmark search query request
#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListRequest {
    /// A search of criteria, restricting the set of bookmarks that qualify.
    ///
    /// All criteria are merged using logical AND / set intersection.
    #[serde(default)]
    query: Vec<BookmarkSearch>,

    #[serde(flatten)]
    pagination: Option<Pagination>,
}

/// List the user's bookmarks matching a query, newest to oldest
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(post,
    path = "/bookmarks",
    tag = "Bookmarks",
    responses(
        (status = 200, body = inline(ListBookmarkResult), description = "Lists bookmarks matching the tag"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn list_bookmarks_matching(
    mut txn: DbTransaction,
    Json(ListRequest { query, pagination }): Json<ListRequest>,
) -> Result<Json<ListBookmarkResult>, (StatusCode, Json<ApiError>)> {
    let ListResult { batch, next_cursor } = list_bookmarks(
        &mut txn,
        &query,
        &pagination.unwrap_or_default(),
    )
    .await
    .map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query for bookmark associations");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    Ok(Json(ListBookmarkResult {
        bookmarks: batch,
        next_cursor,
    }))
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, ToResponse, PartialEq, Eq)]
pub struct BookmarkCreateRequest {
    /// The new bookmark's data. Contrary to the OpenAPI docs, `id` and `user_id` are optional and not used.
    pub bookmark: NewBookmark,

    /// Tags to associate with the bookmark
    #[serde(default)]
    pub tag_names: Vec<String>,

    /// Links to associate with the bookmark
    #[serde(default)]
    pub associations: Vec<AssociatedLink>,
}

/// Create a new bookmark
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(post,
    path = "/bookmark/create",
    tag = "Bookmarks",
    responses(
        (status = 200, body = inline(AnnotatedBookmark), description = "Creates a new bookmark"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn create_bookmark(
    mut txn: DbTransaction<ReadWrite>,
    Json(BookmarkCreateRequest {
        bookmark,
        tag_names,
        associations,
    }): Json<BookmarkCreateRequest>,
) -> Result<Json<AnnotatedBookmark>, (StatusCode, Json<ApiError>)> {
    let tags = txn.ensure_tags(tag_names.as_slice()).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, ?tag_names, "could not ensure tags exist");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    let bookmark = txn.add_bookmark(bookmark).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, ?tag_names, "could not create bookmark");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    for a in &associations {
        let url_id = txn.ensure_url(&a.link).await.map_err(|e| {
            tracing::error!(error=%e, error_debug=?e, ?tag_names, link=?a.link, context=?a.context, "could not create associated link URL");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
        })?;
        txn
            .associate_bookmark_link(&bookmark.id, &url_id, a.context.as_deref())
            .await.map_err(|e| {
                tracing::error!(error=%e, error_debug=?e, ?tag_names, ?url_id, link=?a.link, context=?a.context, "could not associate URL");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
            })?;
    }

    txn.commit().await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, ?tag_names, ?bookmark, "could not commit txn");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e)))
    })?;
    Ok(Json(AnnotatedBookmark {
        bookmark,
        tags,
        associations,
    }))
}
