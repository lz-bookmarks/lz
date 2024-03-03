use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query};

use crate::{BookmarkId, IdType, Transaction};

/// The database ID of a tag.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct TagId(i64);

impl IdType<TagId> for TagId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A named tag, possibly assigned to multiple bookmarks.
///
/// See the section in [Transaction][Transaction#working-with-tags]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug, FromRow)]
pub struct Tag<ID: IdType<TagId>> {
    /// Database identifier of the tag.
    #[sqlx(rename = "tag_id")]
    pub id: ID,

    /// Name of the tag.
    pub name: String,

    /// When the tag was first created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Tag<TagId>> for TagId {
    fn from(val: Tag<TagId>) -> Self {
        val.id
    }
}

/// # Working with [`Tag`]s
impl<'c> Transaction<'c> {
    /// Return all existing tags matching the given names.
    ///
    /// If a tag with a given name doesn't exist, it will be missing
    /// in the returned set. The method [`Transaction::ensure_tags`]
    /// will create any that are missing and return all the matching
    /// tags.
    #[tracing::instrument(skip(self))]
    pub async fn get_tags_with_names<
        T: std::fmt::Debug + IntoIterator<Item = S, IntoIter = C>,
        C: Clone + std::iter::Iterator<Item = S>,
        S: AsRef<str>,
    >(
        &mut self,
        tags: T,
    ) -> Result<Vec<Tag<TagId>>, sqlx::Error> {
        let tag_iter = tags.into_iter();
        // Hyper-yikes: sqlx with sqlite does not support WHERE..IN
        // query value interpolation
        // (https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-do-a-select--where-foo-in--query).
        //
        // Here's a gross way to do it, from reddit
        // (https://www.reddit.com/r/rust/comments/15v4035/comment/jwtedbe/):
        // You manually format the query, and place as many ?
        // placeholders in it as there are values, then bind them in a
        // loop. Ugh, but it seems to do the trick (and is safe).
        let tag_placeholders = tag_iter
            .clone()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ");
        let sql = format!(r#"SELECT * FROM tags WHERE name IN ({})"#, tag_placeholders);
        let mut existing_query = sqlx::query_as(&sql);
        for tag in tag_iter {
            existing_query = existing_query.bind(tag.as_ref().to_string());
        }
        let existing_tags: Vec<Tag<TagId>> = existing_query.fetch_all(&mut *self.txn).await?;

        Ok(existing_tags)
    }

    /// Ensure all the tags with the given name exist and return them.
    ///
    /// This method is the ad-hoc-creating mirror to
    /// [`Transaction::get_tags_with_names`]. Use `ensure_tags` to
    /// ensure all the tags with the given name exist in the database.
    #[tracing::instrument(skip(self))]
    pub async fn ensure_tags<
        T: std::fmt::Debug + IntoIterator<Item = S, IntoIter = C>,
        C: std::fmt::Debug + Clone + std::iter::Iterator<Item = S>,
        S: AsRef<str>,
    >(
        &mut self,
        tags: T,
    ) -> Result<Vec<Tag<TagId>>, sqlx::Error> {
        let tag_iter = tags.into_iter();
        let existing_tags = self.get_tags_with_names(tag_iter.clone()).await?;
        let existing_names: BTreeSet<_> = existing_tags.iter().map(|t| t.name.clone()).collect();
        let all_tags: BTreeSet<String> = tag_iter.map(|t| t.as_ref().to_string()).collect();
        let missing_names = all_tags.difference(&existing_names);
        let mut inserted = vec![];
        tracing::info!(?existing_names, "Ensuring tags");
        for name in missing_names {
            let tag = sqlx::query_as(
                r#"
                  INSERT INTO tags (name, created_at) VALUES (?, datetime()) RETURNING *;
                "#,
            )
            .bind(name)
            .fetch_one(&mut *self.txn)
            .await?;
            inserted.push(tag);
        }

        Ok(existing_tags
            .into_iter()
            .chain(inserted.into_iter())
            .collect())
    }
}

/// A named tag, possibly assigned to multiple bookmarks.
///
/// See the section in [Transaction][Transaction#working-with-tags]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow)]
pub struct BookmarkTag<TID: IdType<TagId>, BID: IdType<BookmarkId>> {
    /// Database identifier of the tag.
    pub tag_id: TID,

    pub bookmark_id: BID,
}

/// # Working with a `Bookmark`'s `Tag`s
impl<'c> Transaction<'c> {
    /// Set the tags on a bookmark.
    ///
    /// Any existing tagging will be removed and replaced with the
    /// given set of tags. Tags are not garbage-collected and will
    /// stick around, so they are available for re-use.
    pub async fn set_bookmark_tags<TS, T>(
        &mut self,
        bookmark_id: BookmarkId,
        tags: TS,
    ) -> Result<(), sqlx::Error>
    where
        TS: std::fmt::Debug + IntoIterator<Item = T>,
        T: Into<TagId>,
    {
        let me = self.user().id;
        query!(
            r#"
              DELETE FROM bookmark_tags WHERE bookmark_id = (
                SELECT bookmark_id FROM bookmarks where bookmark_id = ? AND user_id = ?
              );
            "#,
            bookmark_id,
            me,
        )
        .execute(&mut *self.txn)
        .await?;

        self.add_bookmark_tags(bookmark_id, tags).await
    }

    /// Retrieve a bookmark's tags.
    pub async fn get_bookmark_tags(
        &mut self,
        bookmark_id: BookmarkId,
    ) -> Result<Vec<Tag<TagId>>, sqlx::Error> {
        sqlx::query_as(
            r#"
              SELECT tags.*
              FROM
                tags
                JOIN bookmark_tags USING (tag_id)
              WHERE
                bookmark_id = ?
              ORDER BY tags.name;
            "#,
        )
        .bind(bookmark_id)
        .fetch_all(&mut *self.txn)
        .await
    }

    pub async fn add_bookmark_tags<TS, T>(
        &mut self,
        bookmark_id: BookmarkId,
        tags: TS,
    ) -> Result<(), sqlx::Error>
    where
        TS: std::fmt::Debug + IntoIterator<Item = T>,
        T: Into<TagId>,
    {
        for tag in tags {
            let tag_id = tag.into();
            query!(
                r#"
              INSERT INTO bookmark_tags (
                bookmark_id, tag_id
              ) VALUES (?, ?)
              ON CONFLICT DO NOTHING
            "#,
                bookmark_id,
                tag_id,
            )
            .execute(&mut *self.txn)
            .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::*;
    use anyhow::*;
    use sqlx::SqlitePool;
    use url::Url;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_tag(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin_for_user("tester").await?;
        let inserted = txn
            .ensure_tags(["hi", "test", "welp"])
            .await
            .context("ensuring first set of tags")?;
        assert_eq!(inserted.len(), 3);

        let inserted = txn
            .ensure_tags(["hi", "test", "new"])
            .await
            .context("ensuring second set of tags")?;
        assert_eq!(inserted.len(), 3);

        let existing = txn
            .get_tags_with_names(["welp", "new"])
            .await
            .context("retrieving tags")?;
        assert_eq!(
            existing
                .into_iter()
                .map(|t| t.name)
                .collect::<BTreeSet<String>>(),
            BTreeSet::from(["welp".to_string(), "new".to_string()])
        );

        txn.commit().await?;
        Ok(())
    }

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn bookmark_tags(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin_for_user("tester").await?;
        let tags = txn.ensure_tags(["hi", "test"]).await?;
        let bookmark_id = txn
            .add_bookmark(Bookmark {
                id: (),
                user_id: (),
                created_at: Default::default(),
                modified_at: None,
                accessed_at: None,
                url: Url::parse("https://github.com/antifuchs/lz")?,
                title: "The lz repo".to_string(),
                description: Some("Our extremely high-quality repo".to_string()),
                website_title: None,
                website_description: None,
                notes: Some("".to_string()),
                import_properties: None,
                shared: false,
                unread: false,
            })
            .await?;
        let other_bookmark_id = txn
            .add_bookmark(Bookmark {
                id: (),
                user_id: (),
                created_at: Default::default(),
                modified_at: None,
                accessed_at: None,
                url: Url::parse("https://github.com/antifuchs/governor")?,
                title: "The governor repo".to_string(),
                description: Some("Another extremely high-quality repo".to_string()),
                website_title: None,
                website_description: None,
                notes: Some("".to_string()),
                import_properties: None,
                shared: false,
                unread: false,
            })
            .await?;
        let other_tags = txn.ensure_tags(["welp", "not-this"]).await?;

        txn.set_bookmark_tags(bookmark_id, tags)
            .await
            .context("Setting tags on the bookmark")?;
        txn.set_bookmark_tags(other_bookmark_id, other_tags)
            .await
            .context("Setting other tags on the other bookmark")?;

        let existing_tags = txn
            .get_bookmark_tags(bookmark_id)
            .await
            .context("Retrieving tags")?;
        let existing_other_tags = txn
            .get_bookmark_tags(other_bookmark_id)
            .await
            .context("Retrieving tags")?;
        assert_eq!(
            existing_tags
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<&str>>(),
            vec!["hi", "test"]
        );
        assert_eq!(
            existing_other_tags
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<&str>>(),
            vec!["not-this", "welp"]
        );
        Ok(())
    }
}
