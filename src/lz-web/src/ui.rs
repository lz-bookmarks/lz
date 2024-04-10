//! Web UI rendering routes.

use std::sync::Arc;

use askama_axum::Template;
use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use lz_db::IdType as _;
use lz_db::{BookmarkId, BookmarkSearch};
use tower_http::cors::CorsLayer;

use crate::api::{AnnotatedBookmark, Pagination};
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
    let per_page = pagination.per_page.unwrap_or(20);
    let user_id = txn.user().id;
    let query = vec![];
    let bms = txn
        .list_bookmarks_matching(
            [&[BookmarkSearch::User { id: user_id }], query.as_slice()].concat(),
            per_page,
            pagination.cursor,
        )
        .await
        .map_err(|e| ())?;
    let mut taggings = txn.tags_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query tags for bookmarks");
        ()
    })?;
    let mut associations = txn.associated_links_on_bookmarks(&bms).await.map_err(|e| {
        tracing::error!(error=%e, error_debug=?e, "could not query for bookmark associations");
        ()
    })?;
    let mut next_cursor = None;
    let mut batch = vec![];
    for (elt, bm) in bms.into_iter().enumerate() {
        if elt == usize::from(per_page) {
            // The "next cursor" element:
            next_cursor = Some(bm.id);
            break;
        }
        let id = bm.id;
        if let Some(tags) = taggings.remove(&id) {
            batch.push(AnnotatedBookmark {
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
    Ok(htmz
        .build()
        .title("My bookmarks")
        .wrap(MyBookmarks { batch, next_cursor }))
}
