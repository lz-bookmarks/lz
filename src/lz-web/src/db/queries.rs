use lz_db::{BookmarkId, BookmarkSearch};

use crate::api::{AnnotatedBookmark, Pagination};

use super::DbTransaction;

pub struct ListResult {
    pub next_cursor: Option<BookmarkId>,
    pub batch: Vec<AnnotatedBookmark>,
}

pub async fn list_bookmarks(
    txn: &mut DbTransaction,
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
    let mut taggings = txn.tags_on_bookmarks(&bms).await?;
    let mut associations = txn.associated_links_on_bookmarks(&bms).await?;
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
    Ok(ListResult { next_cursor, batch })
}
