//! The lz-web API
//!
//! We use OpenAPI via the [utoipa] crate to generate an OpenAPI spec.

use std::sync::Arc;

use axum::{debug_handler, extract::Query, http::StatusCode, routing::get, Json, Router};
use axum_valid::Valid;
use lz_db::{BookmarkId, ExistingBookmark, ExistingTag, IdType as _, UserId};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToResponse, ToSchema};
use validator::Validate;

use crate::db::{DbTransaction, GlobalWebAppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "Bookmarks", description = "Managing one's bookmarks")),
    paths(list_bookmarks),
    security(),
    servers((url = "/api/v1/")),
    components(
        schemas(UserId, BookmarkId, ExistingBookmark, ExistingTag, Pagination),
        responses(AnnotatedBookmark, UserId, ExistingBookmark, ExistingTag)
    )
)]
pub struct ApiDoc;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    Router::new()
        .route("/bookmarks", get(list_bookmarks))
        .layer(CorsLayer::permissive())
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                    )
                    .on_response(
                        tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                    ),
            ),
        )
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
/// the previous page's last element was. To that end, fill the
/// `last_seen` parameter.
#[derive(
    Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq, Hash, Validate, ToSchema,
)]
#[schema(default)]
struct Pagination {
    /// The last batch's last (oldest) bookmark ID
    #[schema(example = None)]
    last_seen: Option<BookmarkId>,

    /// How many items to return
    #[schema(example = 50)]
    #[validate(range(min = 1, max = 500))]
    per_page: Option<u16>,
}

/// A bookmark, including tags set on it.
#[derive(Serialize, Debug, ToSchema, ToResponse)]
pub struct AnnotatedBookmark {
    bookmark: ExistingBookmark,
    tags: Vec<ExistingTag>,
}

/// List the user's bookmarks, newest to oldest.
#[debug_handler(state = Arc<GlobalWebAppState>)]
#[utoipa::path(get,
    path = "/bookmarks",
    tag = "Bookmarks",
    params(
        ("pagination" = inline(Option<Pagination>),
            Query,
            style = Form,
            explode,
        ),
    ),
    responses(
        (status = 200, body = inline(Vec<AnnotatedBookmark>), description = "Lists all bookmarks"),
    ),
)]
#[tracing::instrument(skip(txn))]
async fn list_bookmarks(
    mut txn: DbTransaction,
    pagination: Option<Valid<Query<Pagination>>>,
) -> Result<Json<Vec<AnnotatedBookmark>>, (StatusCode, Json<ApiError>)> {
    let pagination = pagination.unwrap_or_default();
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
