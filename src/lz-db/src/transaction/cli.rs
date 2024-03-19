//! CLI-specific queries

use sqlx;

use crate::{Bookmark, BookmarkId, Transaction, UserId};

/// # Queries relevant to the `lz` CLI application
impl Transaction {
    /// Retrieve all the current user's bookmarks
    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(self))]
    // TODO: Can we not call this `list_bookmarks` due to the name overlap?
    // Is there any use to this level of tracing on the CLI?
    pub async fn all_bookmarks(
        &mut self,
        limit: Option<u16>,
    ) -> Result<Vec<Bookmark<BookmarkId, UserId>>, sqlx::Error> {
        let mut query = sqlx::QueryBuilder::new(
            r#"
              SELECT bookmarks.* FROM bookmarks
              WHERE
                user_id = "#,
        );
        query.push_bind(self.user().id).push(
            r#"
              ORDER BY
                created_at DESC, bookmark_id DESC
            "#,
        );
        if let Some(max_count) = limit {
            query.push(" LIMIT ");
            query.push_bind(max_count);
        }
        query
            .build_query_as::<Bookmark<BookmarkId, UserId>>()
            .fetch_all(&mut *self.txn)
            .await
    }
}
