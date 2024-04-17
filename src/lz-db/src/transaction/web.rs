//! Web-app specific transactional queries

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use sqlx::QueryBuilder;
use url::Url;
use utoipa::{ToResponse, ToSchema};

use crate::{
    Bookmark, BookmarkId, BookmarkSearch, BookmarkSearchCriteria, Tag, TagId, Transaction,
    TransactionMode, UserId,
};

/// # Queries relevant to the `lz` web app
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
impl<M: TransactionMode> Transaction<M> {
    /// Retrieve bookmarks tagged matching the given criteria, paginated
    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(self))]
    pub async fn list_bookmarks_matching(
        &mut self,
        criteria: &[BookmarkSearch],
        page_size: u16,
        last_seen: Option<BookmarkId>,
    ) -> Result<Vec<Bookmark<BookmarkId, UserId>>, sqlx::Error> {
        let mut qb = QueryBuilder::new(
            "SELECT bookmarks.*, urls.link AS url FROM bookmarks JOIN urls USING (url_id)",
        );

        // Limit the bookmarks by the relationships they have: For
        // tags, we handle that by finding each tag's bookmark IDs and
        // intersecting them. This _seems_ like it ought to be
        // inefficient, but at "normal" numbers of bookmarks and tags,
        // sqlite can get a pretty fast query plan out of it.
        qb.push(" JOIN (");
        let mut sep = qb.separated(" INTERSECT ");
        for criterium in criteria.iter() {
            sep = criterium.bookmarks_join_table(sep);
        }
        // A query for "all" bookmarks to ensure the JOIN works
        // even if no criteria were given:
        sep.push("SELECT bookmark_id FROM bookmarks");
        qb.push(") USING (bookmark_id)");

        // Limit the bookmarks by any "additional" criteria that might
        // apply (creation, user ID, and of course, pagination):
        qb.push(" WHERE ");
        if let Some(last_seen) = last_seen {
            qb.push("created_at <= (SELECT created_at FROM bookmarks WHERE bookmark_id = ");
            qb.push_bind(last_seen);
            qb.push(") ");
            qb.push(" AND ");
        }
        if !criteria.is_empty() {
            qb.push("(");
            let mut sep = qb.separated(") AND (");
            for criterium in criteria.iter() {
                sep = criterium.where_clause(sep);
            }
            qb.push(")");
        } else {
            qb.push("1=1");
        }
        qb.push(" ORDER BY created_at DESC, bookmark_id DESC LIMIT ");
        qb.push_bind(page_size + 1);

        tracing::debug!(sql = qb.sql());
        qb.build_query_as().fetch_all(&mut *self.txn).await
    }

    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(self))]
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

    pub async fn tags_matching(
        &mut self,
        tag_fragment: &str,
    ) -> Result<Vec<Tag<TagId>>, sqlx::Error> {
        sqlx::query_as(
            r#"
          SELECT * from tags WHERE name LIKE ?
        "#,
        )
        .bind(format!("%{tag_fragment}%"))
        .fetch_all(&mut *self.txn)
        .await
    }
}

/// A link associated with a bookmark.
///
/// Links can have a "context" in which that association happens
/// (free-form text, given by the user), and they point to a URL,
/// which in turn can be another bookmark.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, FromRow, ToSchema, ToResponse)]
pub struct AssociatedLink {
    pub context: Option<String>,
    #[sqlx(try_from = "&'a str")]
    pub link: Url,
    // TODO: query for the bookmark on the right side, if one exists.
}

impl<M: TransactionMode> Transaction<M> {
    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), skip(self))]
    pub async fn associated_links_on_bookmarks<
        I: IntoIterator<Item = B, IntoIter = C> + Clone + fmt::Debug,
        C: Clone + std::iter::Iterator<Item = B>,
        B: Into<BookmarkId>,
    >(
        &mut self,
        bms: I,
    ) -> Result<HashMap<BookmarkId, Vec<AssociatedLink>>, sqlx::Error> {
        let mut qb = QueryBuilder::new(
            r#"
              SELECT bookmark_associations.bookmark_id AS bookmark_id, urls.link AS link, bookmark_associations.context AS context
              FROM bookmark_associations JOIN urls USING (url_id)
              WHERE bookmark_associations.bookmark_id IN
            "#,
        );
        qb.push_tuples(bms.into_iter(), |mut b, bm| {
            b.push_bind(bm.into());
        });
        qb.push(" ORDER BY bookmark_id");

        #[derive(FromRow)]
        struct AssociationFromBookmark {
            bookmark_id: BookmarkId,
            #[sqlx(flatten)]
            association: AssociatedLink,
        }
        let result: Vec<AssociationFromBookmark> =
            qb.build_query_as().fetch_all(&mut *self.txn).await?;
        let mut associations = HashMap::new();
        for a in result {
            associations
                .entry(a.bookmark_id)
                .or_insert(vec![])
                .push(a.association);
        }
        Ok(associations)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context as _;
    use test_context::test_context;
    use testresult::TestResult;
    use url::Url;

    use crate::testing::Context;
    use crate::*;

    #[test_context(Context)]
    #[tokio::test]
    async fn paginate_bookmarks(ctx: &mut Context) -> TestResult {
        let mut txn = ctx.begin().await?;

        let bookmark_count = 60; // how many to generate
        let page_size = 50; // how many to retrieve in a batch

        let mut reference_time = chrono::DateTime::default()
            .checked_sub_days(chrono::Days::new(bookmark_count))
            .unwrap();
        for i in 0..bookmark_count - 1 {
            let bookmark = Bookmark {
                id: NoId,
                user_id: NoId,
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
        // insert a backdated bookmark (simulating an import):
        reference_time = reference_time
            .checked_sub_days(chrono::Days::new(bookmark_count * 2))
            .unwrap();
        let backdated = Bookmark {
            id: NoId,
            user_id: NoId,
            created_at: reference_time,
            modified_at: Some(Default::default()),
            accessed_at: Some(Default::default()),
            url: Url::parse("https://github.com/antifuchs/lz?key=backdated")?,
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
        let backdated = txn
            .add_bookmark(backdated.clone())
            .await
            .with_context(|| "adding backdated bookmark".to_string())?;

        let bookmarks_batch_1 = txn
            .list_bookmarks_matching(&vec![], page_size, None)
            .await?;
        assert_eq!(bookmarks_batch_1.len(), (page_size + 1) as usize);

        let bookmarks_batch_2 = txn
            .list_bookmarks_matching(&vec![], page_size, bookmarks_batch_1.last().map(|bm| bm.id))
            .await?;
        assert_eq!(bookmarks_batch_2.len(), 10);
        assert_eq!(bookmarks_batch_2.last().map(|bm| bm.id), Some(backdated.id));
        Ok(())
    }
}
