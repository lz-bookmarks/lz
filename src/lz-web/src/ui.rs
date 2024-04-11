//! Web UI rendering routes.

use std::sync::Arc;

use askama_axum::Template;
use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use lz_db::BookmarkId;
use lz_db::IdType as _;
use tower_http::cors::CorsLayer;

use crate::api::{AnnotatedBookmark, Pagination};
use crate::db::queries::{list_bookmarks, ListResult};
use crate::db::{DbTransaction, GlobalWebAppState};

mod htmz;
use htmz::*;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    let router = Router::new()
        .route("/", get(my_bookmarks))
        .layer(CorsLayer::permissive());
    router
}

#[derive(Template)]
#[template(path = "my_bookmarks.html", ext = "html")]
struct MyBookmarks {
    batch: Vec<AnnotatedBookmark>,
    next_cursor: Option<BookmarkId>,
}

#[tracing::instrument()]
async fn my_bookmarks(
    mut txn: DbTransaction,
    Query(pagination): Query<Pagination>,
    htmz: HtmzMode,
) -> Result<HtmzTemplate<MyBookmarks>, ()> {
    let ListResult { batch, next_cursor } = list_bookmarks(&mut txn, &vec![], &pagination)
        .await
        .map_err(|error| {
            tracing::error!(?error, %error, ?txn, "Could not query for bookmarks");
        })?;
    Ok(htmz
        .build()
        .title("My bookmarks")
        .wrap(MyBookmarks { batch, next_cursor }))
}
