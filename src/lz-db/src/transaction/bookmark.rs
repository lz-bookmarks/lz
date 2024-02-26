use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query_scalar, types::Text};
use url::Url;

use crate::{IdType, Transaction};

/// The database ID of a bookmark.
#[derive(PartialEq, Eq, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct BookmarkId(i64);

impl IdType<BookmarkId> for BookmarkId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A bookmark saved by a user.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub struct Bookmark<ID: IdType<BookmarkId>> {
    #[sqlx(rename = "bookmark_id")]
    pub id: ID,

    /// URL that the bookmark points to.
    #[sqlx(try_from = "&'a str")]
    pub url: Url,

    /// Title that the user gave the bookmark.
    pub title: String,

    /// Description of the bookmark, possibly extracted from the website.
    pub description: String,

    /// Original title extracted from the website.
    pub website_title: Option<String>,

    /// Original description extracted from the website.
    pub website_description: Option<String>,

    /// Private notes that the user attached to the bookmark.
    pub notes: String,
}

/// # Working with Bookmarks
impl<'c> Transaction<'c> {
    /// Store a new bookmark in the database.
    pub async fn add_bookmark(&mut self, bm: Bookmark<()>) -> Result<BookmarkId, sqlx::Error> {
        let bm_url = Text(bm.url);
        let id = query_scalar!(
            r#"
              INSERT INTO bookmarks (
                url,
                title,
                description,
                website_title,
                website_description,
                notes
              ) VALUES (?, ?, ?, ?, ?, ?)
              RETURNING bookmark_id;
            "#,
            bm_url,
            bm.title,
            bm.description,
            bm.website_title,
            bm.website_description,
            bm.notes,
        )
        .fetch_one(&mut *self.txn)
        .await?;
        Ok(BookmarkId(id))
    }

    /// Retrieve the bookmark with the given ID.
    pub async fn get_bookmark_by_id(
        &mut self,
        id: i64,
    ) -> Result<Bookmark<BookmarkId>, sqlx::Error> {
        sqlx::query_as(
            r#"
               SELECT * FROM bookmarks WHERE bookmark_id = ?;
            "#,
        )
        .bind(id)
        .fetch_one(&mut *self.txn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use sqlx::SqlitePool;
    use url::Url;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_bookmark(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let to_add = Bookmark {
            id: (),
            url: Url::parse("https://github.com/antifuchs/lz")?,
            title: "The lz repo".to_string(),
            description: "This is a great repo with excellent code.".to_string(),
            website_title: Some("lz, the bookmarks manager".to_string()),
            website_description: Some(
                "Please do not believe in the quality of this code.".to_string(),
            ),
            notes: "No need to run tests.".to_string(),
        };
        let mut txn = conn.begin().await?;
        let added = txn.add_bookmark(to_add.clone()).await?;
        let retrieved = txn.get_bookmark_by_id(added.id()).await?;
        txn.commit().await?;

        assert_eq!(added, retrieved.id);
        assert_eq!(
            to_add,
            Bookmark::<()> {
                id: (),
                url: retrieved.url,
                title: retrieved.title,
                description: retrieved.description,
                website_title: retrieved.website_title,
                website_description: retrieved.website_description,
                notes: retrieved.notes,
            }
        );
        Ok(())
    }
}
