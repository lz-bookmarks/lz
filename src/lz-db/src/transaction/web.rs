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
    /// The pagination mechanism works according to the cursor
    /// principle: Instead of a offset/limit, the web app passes a
    /// "next" ID (the highest-ID bookmark that would be
    /// eligible). That results in an indexed query that doesn't have
    /// to traverse arbitrary numbers of potential results.
    ///
    /// To ensure the web app can tell that there is a next batch,
    /// this function returns one more element than was requested. If
    /// page_size+1 elements are returned, that last element's ID
    /// should be the next cursor ID.
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
                AND bookmark_id <= ?
              ORDER BY
                created_at DESC, bookmark_id DESC
              LIMIT ?
            "#,
        )
        .bind(self.user().id)
        .bind(last_seen)
        .bind(page_size + 1)
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
                .or_insert_with(Vec::new)
                .push(bmt.tag);
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context as _;
    use sqlx::SqlitePool;
    use url::Url;

    use crate::*;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn paginate_bookmarks(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin_for_user("tester").await?;

        let bookmark_count = 60; // how many to generate
        let page_size = 50; // how many to retrieve in a batch

        let mut reference_time = chrono::DateTime::default()
            .checked_sub_days(chrono::Days::new(bookmark_count))
            .unwrap();
        for i in 0..bookmark_count {
            let bookmark = Bookmark {
                id: (),
                user_id: (),
                created_at: reference_time,
                modified_at: Some(Default::default()),
                accessed_at: Some(Default::default()),
                url: Url::parse(&format!("https://github.com/antifuchs/lz?key={i}"))?,
                title: "The lz repo".to_string(),
                description: Some("This is a great repo with excellent code.".to_string()),
                website_title: Some("lz, the bookmarks manager".to_string()),
                website_description: Some(
                    "Please do not believe in the quality of this code.".to_string(),
                ),
                notes: Some("No need to run tests.".to_string()),
                import_properties: None,
                shared: true,
                unread: true,
            };
            reference_time = reference_time
                .checked_add_days(chrono::Days::new(1))
                .unwrap();
            txn.add_bookmark(bookmark.clone())
                .await
                .with_context(|| format!("adding bookmark {i}"))?;
        }
        let bookmarks_batch_1 = txn.list_bookmarks(page_size, None).await?;
        assert_eq!(bookmarks_batch_1.len(), (page_size + 1) as usize);

        let bookmarks_batch_2 = txn
            .list_bookmarks(page_size, bookmarks_batch_1.last().map(|bm| bm.id))
            .await?;
        assert_eq!(bookmarks_batch_2.len(), 10);
        Ok(())
    }
}
