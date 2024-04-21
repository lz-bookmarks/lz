//! The lz-web API
//!
//! We use OpenAPI via the [utoipa] crate to generate an OpenAPI spec.

mod observability;
mod searching;

use std::sync::Arc;

use axum::extract::Query;
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use lz_db::{
    AssociatedLink, BookmarkId, BookmarkSearch, BookmarkSearchDateParams,
    BookmarkSearchDatetimeField, BookmarkSearchDatetimeOrientation, DateInput, ExistingBookmark,
    ExistingTag, NewBookmark, NoId, ReadWrite, TagId, TagName, UserId,
};
use searching::TagQuery;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use url::Url;
use utoipa::{IntoParams, OpenApi, ToResponse, ToSchema};

use crate::db::queries::{list_bookmarks, AnnotatedBookmark, ListResult, Pagination};
use crate::db::{DbTransaction, GlobalWebAppState};
use crate::http::{lookup_page_from_web, Metadata};

mod error;
use error::ApiError;

#[derive(OpenApi)]
#[openapi(
    tags((name = "Bookmarks", description = "Managing one's bookmarks")),
    paths(list_bookmarks_matching, create_bookmark, complete_tag, fetch_page_metadata),
    security(),
    servers((url = "/api/v1/")),
    components(
        schemas(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, BookmarkId, ExistingBookmark, ExistingTag, Pagination, TagName, TagQuery, ListRequest, BookmarkSearch, BookmarkSearchDateParams, DateInput, BookmarkSearchDatetimeField, BookmarkSearchDatetimeOrientation, TagId, NoId, BookmarkCreateRequest, Metadata),
        responses(ListBookmarkResult, AnnotatedBookmark, AssociatedLink, UserId, ExistingBookmark, ExistingTag)
    )
)]
pub struct ApiDoc;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    let router = Router::new()
        .route("/bookmarks", post(list_bookmarks_matching))
        .route("/bookmark/create", post(create_bookmark))
        .route("/http/fetch_metadata", get(fetch_page_metadata))
        .route("/tag/complete", get(complete_tag))
        .layer(CorsLayer::permissive());
    observability::add_layers(router)
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
) -> Result<Json<ListBookmarkResult>, ApiError> {
    let ListResult { batch, next_cursor } =
        list_bookmarks(&mut txn, &query, &pagination.unwrap_or_default()).await?;
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
) -> Result<Json<AnnotatedBookmark>, ApiError> {
    let tags = txn.ensure_tags(tag_names.as_slice()).await?;
    let bookmark = txn.add_bookmark(bookmark).await?;
    for a in &associations {
        let url_id = txn.ensure_url(&a.link).await?;
        txn.associate_bookmark_link(&bookmark.id, &url_id, a.context.as_deref())
            .await?;
    }

    txn.commit().await?;
    Ok(Json(AnnotatedBookmark {
        bookmark,
        tags,
        associations,
    }))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct CompleteQuery {
    tag_fragment: String,
}

#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get,
    path = "/tag/complete",
    params(("tag_fragment" = String, Query, description = "Substring of the tag name that must match")),
    tag = "Tags",
    responses(
        (status = 200, body = inline(Vec<ExistingTag>), description = "Return tags for autocompletion"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn complete_tag(
    mut txn: DbTransaction,
    Query(CompleteQuery { tag_fragment }): Query<CompleteQuery>,
) -> Result<Json<Vec<ExistingTag>>, ApiError> {
    Ok(Json(txn.tags_matching(&tag_fragment).await?))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct PageMetadataQuery {
    url: Url,
}

#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get,
    path = "/http/fetch_metadata",
    params(("url" = String, Query, description = "URL to retrieve and inspect for metadata")),
    tag = "HTTP",
    responses(
        (status = 200, body = inline(Metadata), description = "Returns page metadata"),
    ),
)]
#[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(txn))]
async fn fetch_page_metadata(
    mut txn: DbTransaction<ReadWrite>,
    Query(PageMetadataQuery { url }): Query<PageMetadataQuery>,
) -> Result<Json<Metadata>, ApiError> {
    txn.ensure_url(&url).await?;
    txn.commit().await?;
    Ok(Json(lookup_page_from_web(&url).await?))
}
