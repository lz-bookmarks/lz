//! Web-app specific transactional queries

use crate::{Bookmark, BookmarkId, IdType, Transaction, UserId};

/// # Queries relevant to the `lz` web app
impl Transaction {
    /// Retrieve a user's bookmarks from the database, paginated
    ///
    /// ## Pagination
    ///
    /// The pagination mechanism works according to the "last seen"
    /// principle: Instead of a offset/limit, the web app passes a
    /// "last seen" ID (the last bookmark in the previous batch). That
    /// results in an indexed query that doesn't have to traverse
    /// arbitrary numbers of potential results.
    #[tracing::instrument(skip(self))]
    pub async fn list_bookmarks(
        &mut self,
        page_size: u16,
        last_seen: Option<BookmarkId>,
    ) -> Result<Vec<Bookmark<BookmarkId, UserId>>, sqlx::Error> {
        let last_seen = last_seen.map(|id| id.id()).unwrap_or(i64::MAX);
        sqlx::query_as(
            r#"
              SELECT bookmarks.* FROM bookmarks
              WHERE
                user_id = ?
                AND bookmark_id < ?
              ORDER BY
                created_at, bookmark_id
              LIMIT ?
            "#,
        )
        .bind(self.user().id)
        .bind(last_seen)
        .bind(page_size)
        .fetch_all(&mut *self.txn)
        .await
    }
}
