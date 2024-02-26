use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use crate::{IdType, Transaction};

/// The database ID of a tag.
#[derive(PartialEq, Eq, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct TagId(i64);

impl IdType<TagId> for TagId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A named tag, possibly assigned to multiple bookmarks.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow)]
pub struct Tag<ID: IdType<TagId>> {
    /// Database identifier of the tag.
    pub tag_id: ID,

    /// Name of the tag.
    pub name: String,
}

/// # Working with [`Tag`]s
impl<'c> Transaction<'c> {
    /// Return all existing tags matching the given names.
    ///
    /// If a tag with a given name doesn't exist, it will be missing
    /// in the returned set. The method [`ensure_tags`] will create
    /// any that are missing and return all the matching tags.
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
    /// [`get_tags_with_names`]. Use `ensure_tags` to ensure all the
    /// tags with the given name exist in the database.
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
                  INSERT INTO tags (name) VALUES (?) RETURNING *;
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::*;
    use sqlx::SqlitePool;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_tag(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin().await?;
        let inserted = txn.ensure_tags(["hi", "test", "welp"]).await?;
        assert_eq!(inserted.len(), 3);

        let inserted = txn.ensure_tags(["hi", "test", "new"]).await?;
        assert_eq!(inserted.len(), 3);

        let existing = txn.get_tags_with_names(["welp", "new"]).await?;
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
}
