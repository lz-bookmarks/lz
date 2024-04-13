//! Web UI rendering routes.

use std::sync::Arc;

use askama_axum::Template;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Router};
use lz_db::{AssociatedLink, Bookmark, BookmarkId, ExistingTag, IdType, NoId, ReadWrite, UserId};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use url::Url;

use crate::db::queries::{
    annotate_bookmarks, list_bookmarks, AnnotatedBookmark, ListResult, Pagination,
};
use crate::db::{DbTransaction, GlobalWebAppState};

mod htmz;
use htmz::*;

pub fn router() -> Router<Arc<GlobalWebAppState>> {
    Router::new()
        .route("/", get(my_bookmarks))
        .route("/edit", get(bookmark_edit_form))
        .route("/edit", post(bookmark_update))
        .route("/new", get(bookmark_create_form))
        .layer(CorsLayer::permissive())
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
    let ListResult { batch, next_cursor } = list_bookmarks(&mut txn, &[], &pagination)
        .await
        .map_err(|error| {
            tracing::error!(?error, %error, ?txn, "Could not query for bookmarks");
        })?;
    Ok(htmz
        .build()
        .title("My bookmarks")
        .wrap(MyBookmarks { batch, next_cursor }))
}

#[derive(Template)]
#[template(path = "bookmark_edit_form.html", ext = "html")]
struct BookmarkEditForm<ID: IdType<BookmarkId>, UID: IdType<UserId>> {
    bookmark: Bookmark<ID, UID>,
    tags: Vec<ExistingTag>,
    associations: Vec<AssociatedLink>,
}

impl From<AnnotatedBookmark> for BookmarkEditForm<BookmarkId, UserId> {
    fn from(value: AnnotatedBookmark) -> Self {
        Self {
            bookmark: value.bookmark,
            tags: value.tags,
            associations: value.associations,
        }
    }
}

impl From<Bookmark<NoId, NoId>> for BookmarkEditForm<NoId, NoId> {
    fn from(value: Bookmark<NoId, NoId>) -> Self {
        Self {
            bookmark: value,
            tags: vec![],
            associations: vec![],
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct BookmarkEditFormParams {
    id: BookmarkId,
}

#[tracing::instrument()]
async fn bookmark_edit_form(
    mut txn: DbTransaction,
    Query(BookmarkEditFormParams { id }): Query<BookmarkEditFormParams>,
    htmz: HtmzMode,
) -> Result<HtmzTemplate<BookmarkEditForm<BookmarkId, UserId>>, ()> {
    let bm = txn.get_bookmark_by_id(id.id()).await.map_err(|error| {
        tracing::error!(?error, %error, "Could not get bookmark");
    })?;
    let id = bm.id;
    let (mut annotated, _) = annotate_bookmarks(&mut txn, &[bm], 1)
        .await
        .map_err(|error| {
            tracing::error!(?error, %error, "Could not annotate bookmark");
        })?;
    Ok(htmz
        .build()
        .title(format!("Editing bookmark {:?}", id))
        .wrap(annotated.remove(0).into()))
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct BookmarkCreateFormParams {
    url: Url,
}

#[tracing::instrument()]
async fn bookmark_create_form(
    mut txn: DbTransaction,
    Query(BookmarkCreateFormParams { url }): Query<BookmarkCreateFormParams>,
    htmz: HtmzMode,
) -> Result<axum::response::Response, ()> {
    if let Some(existing) = txn.find_bookmark_with_url(&url).await.map_err(|error| {
        tracing::error!(?error, %error, "could not query for existing bookmark");
    })? {
        let (mut annotated, _) =
            annotate_bookmarks(&mut txn, &[existing], 1)
                .await
                .map_err(|error| {
                    tracing::error!(?error, %error, %url, "could not annotate existing bookmark");
                })?;
        Ok(askama_axum::into_response(
            &htmz
                .build()
                .title(format!("Editing bookmark with URL {:?}", url))
                .wrap(BookmarkEditForm::<BookmarkId, UserId>::from(
                    annotated.remove(0),
                )),
        ))
    } else {
        let new_bookmark = crate::http::lookup_link_from_web(&url)
            .await
            .map_err(|error| {
                tracing::error!(?error, %error, %url, "could not retrieve url");
            })?;
        Ok(askama_axum::into_response(
            &htmz
                .build()
                .title(format!("Adding new bookmark for {}", url))
                .wrap(BookmarkEditForm::from(new_bookmark)),
        ))
    }
}

#[derive(Debug, Serialize, Template)]
#[template(path = "partials/bookmark_item.html")]
struct BookmarkItem {
    item: AnnotatedBookmark,
}

#[tracing::instrument()]
async fn bookmark_update(
    mut txn: DbTransaction<ReadWrite>,
    htmz: HtmzMode,
    Form(data): Form<Bookmark<BookmarkId, UserId>>,
) -> Result<HtmzTemplate<BookmarkItem>, ()> {
    txn.update_bookmark(&data).await.map_err(|error| {
        tracing::error!(?error, %error, ?data, "Could not update bookmark");
    })?;
    let bm = txn
        .get_bookmark_by_id(data.id.id())
        .await
        .map_err(|error| {
            tracing::error!(?error, %error, "Could not get bookmark");
        })?;
    let (mut annotated, _) = annotate_bookmarks(&mut txn, &[bm], 1)
        .await
        .map_err(|error| {
            tracing::error!(?error, %error, "Could not annotate bookmark");
        })?;
    let item = annotated.remove(0);
    txn.commit().await.map_err(|error| {
        tracing::error!(?error, %error, ?data, "Could not commit transaction");
    })?;
    Ok(htmz
        .build()
        .title(format!("Bookmark for {}", item.bookmark.url))
        .wrap(BookmarkItem { item }))
}
