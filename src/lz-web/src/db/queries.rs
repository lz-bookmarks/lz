use lz_db::{
    AssociatedLink, BookmarkId, BookmarkSearch, ExistingBookmark, ExistingTag, TransactionMode,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use super::DbTransaction;

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
pub struct Pagination {
    /// The last batch's last (oldest) bookmark ID
    #[schema(example = None)]
    pub cursor: Option<BookmarkId>,

    /// How many items to return
    #[schema(example = 50)]
    #[validate(range(min = 1, max = 500))]
    pub per_page: Option<u16>,
}

pub struct ListResult {
    pub next_cursor: Option<BookmarkId>,
    pub batch: Vec<AnnotatedBookmark>,
}

pub async fn list_bookmarks<M: TransactionMode>(
    txn: &mut DbTransaction<M>,
    query: &[BookmarkSearch],
    pagination: &Pagination,
) -> Result<ListResult, sqlx::Error> {
    let per_page = pagination.per_page.unwrap_or(20);
    let user_id = txn.user().id;
    let bms = txn
        .list_bookmarks_matching(
            &[&[BookmarkSearch::User { id: user_id }], query].concat(),
            per_page,
            pagination.cursor,
        )
        .await?;
    let (batch, next_cursor) = annotate_bookmarks(txn, &bms, per_page).await?;
    Ok(ListResult { next_cursor, batch })
}

/// A bookmark, including tags and associations on it.
#[derive(Serialize, Debug, ToSchema, ToResponse)]
pub struct AnnotatedBookmark {
    pub bookmark: ExistingBookmark,
    pub tags: Vec<ExistingTag>,
    pub associations: Vec<AssociatedLink>,
}

pub async fn annotate_bookmarks<M: TransactionMode>(
    txn: &mut DbTransaction<M>,
    bookmarks: &[ExistingBookmark],
    per_page: u16,
) -> Result<(Vec<AnnotatedBookmark>, Option<BookmarkId>), sqlx::Error> {
    let mut taggings = txn.tags_on_bookmarks(bookmarks).await?;
    let mut associations = txn.associated_links_on_bookmarks(bookmarks).await?;

    let mut next_cursor = None;
    let mut batch = vec![];
    for (elt, bm) in bookmarks.iter().enumerate() {
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
    Ok((batch, next_cursor))
}
