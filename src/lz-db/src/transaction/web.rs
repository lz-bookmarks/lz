//! Web-app specific transactional queries

use std::{collections::HashMap, fmt};

use sqlx::prelude::*;

use crate::{Bookmark, BookmarkId, IdType, Tag, TagId, Transaction, UserId};

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
                created_at DESC, bookmark_id DESC
              LIMIT ?
            "#,
        )
        .bind(self.user().id)
        .bind(last_seen)
        .bind(page_size)
        .fetch_all(&mut *self.txn)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn tags_on_bookmarks<
        I: IntoIterator<Item = B, IntoIter = C> + Clone + fmt::Debug,
        C: Clone + std::iter::Iterator<Item = B>,
        B: Into<BookmarkId>,
    >(
        &mut self,
        bms: I,
    ) -> Result<HashMap<BookmarkId, Vec<Tag<TagId>>>, sqlx::Error> {
        #[derive(FromRow)]
        struct BMTag {
            bookmark_id: BookmarkId,
            #[sqlx(flatten)]
            tag: Tag<TagId>,
        }
        let bms = bms.into_iter();
        let bm_placeholders = bms.clone().map(|_| "?").collect::<Vec<&str>>().join(", ");
        let sql = format!(
            r#"
              SELECT bookmarks.bookmark_id, tags.tag_id as tag_id, tags.*
              FROM bookmarks
                   JOIN (bookmark_tags JOIN tags USING (tag_id))
                   USING (bookmark_id)
              WHERE bookmarks.user_id = ? AND bookmarks.bookmark_id IN ({})"#,
            bm_placeholders
        );
        let mut query = sqlx::query_as(&sql).bind(self.user().id);
        let mut value = HashMap::new();
        for bm in bms {
            let id = bm.into();
            value.insert(id, Vec::new());
            query = query.bind(id);
        }
        let tags_by_bookmark: Vec<BMTag> = query.fetch_all(&mut *self.txn).await?;
        for bmt in tags_by_bookmark {
            value
                .entry(bmt.bookmark_id)
                .or_insert_with(|| Vec::new())
                .push(bmt.tag);
        }
        Ok(value)
    }
}
