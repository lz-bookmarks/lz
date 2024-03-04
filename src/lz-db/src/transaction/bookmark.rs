use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query_scalar, types::Text};
use url::Url;
use utoipa::{ToResponse, ToSchema};

use crate::{IdType, Transaction, UserId};

/// The database ID of a bookmark.
#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    ToSchema,
    ToResponse,
)]
#[sqlx(transparent)]
pub struct BookmarkId(i64);

impl IdType<BookmarkId> for BookmarkId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A bookmark saved by a user.
///
/// See the section in [Transaction][Transaction#working-with-bookmarks]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, FromRow, ToSchema, ToResponse)]
#[aliases(
    ExistingBookmark = Bookmark<BookmarkId, UserId>,
)]
pub struct Bookmark<ID: IdType<BookmarkId>, UID: IdType<UserId>> {
    /// Database identifier of the bookmark
    #[sqlx(rename = "bookmark_id")]
    pub id: ID,

    /// ID of the user who owns the bookmark
    pub user_id: UID,

    /// Time at which the bookmark was created.
    ///
    /// This time is assigned in code here, not in the database.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last time the bookmark was modified.
    ///
    /// This field indicates modifications to the bookmark data itself
    /// only, not changes to tags or related models.
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last time the bookmark was accessed via the web
    pub accessed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// URL that the bookmark points to.
    #[sqlx(try_from = "&'a str")]
    pub url: Url,

    /// Title that the user gave the bookmark.
    pub title: String,

    /// Description of the bookmark, possibly extracted from the website.
    pub description: Option<String>,

    /// Original title extracted from the website.
    pub website_title: Option<String>,

    /// Original description extracted from the website.
    pub website_description: Option<String>,

    /// Private notes that the user attached to the bookmark.
    pub notes: Option<String>,

    /// Whether the bookmark is "to read"
    pub unread: bool,

    /// Whether other users can see the bookmark.
    pub shared: bool,

    /// Properties imported from other systems.
    #[serde(skip_deserializing)]
    pub import_properties: Option<sqlx::types::Json<crate::ImportProperties>>,
}

impl<U: IdType<UserId>> From<&Bookmark<BookmarkId, U>> for BookmarkId {
    fn from(val: &Bookmark<BookmarkId, U>) -> Self {
        val.id
    }
}

impl<U: IdType<UserId>> From<Bookmark<BookmarkId, U>> for BookmarkId {
    fn from(val: Bookmark<BookmarkId, U>) -> Self {
        val.id
    }
}

/// # Working with Bookmarks
impl Transaction {
    /// Store a new bookmark in the database.
    #[tracing::instrument(skip(self))]
    pub async fn add_bookmark(&mut self, bm: Bookmark<(), ()>) -> Result<BookmarkId, sqlx::Error> {
        let bm_url = Text(bm.url);
        let user_id = self.user().id;
        let id = query_scalar!(
            r#"
              INSERT INTO bookmarks (
                user_id,
                created_at,
                modified_at,
                accessed_at,
                url,
                title,
                description,
                website_title,
                website_description,
                unread,
                shared,
                notes,
                import_properties
              ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
              RETURNING bookmark_id;
            "#,
            user_id,
            bm.created_at,
            bm.modified_at,
            bm.accessed_at,
            bm_url,
            bm.title,
            bm.description,
            bm.website_title,
            bm.website_description,
            bm.unread,
            bm.shared,
            bm.notes,
            bm.import_properties,
        )
        .fetch_one(&mut *self.txn)
        .await?;
        Ok(BookmarkId(id))
    }

    /// Retrieve the bookmark with the given ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_bookmark_by_id(
        &mut self,
        id: i64,
    ) -> Result<Bookmark<BookmarkId, UserId>, sqlx::Error> {
        sqlx::query_as(
            r#"
               SELECT * FROM bookmarks WHERE bookmark_id = ? AND user_id = ?;
            "#,
        )
        .bind(id)
        .bind(self.user().id)
        .fetch_one(&mut *self.txn)
        .await
    }

    /// Find all users' bookmarks with the given URL.
    #[tracing::instrument(skip(self))]
    pub async fn find_bookmarks_by_url_for_everyone(
        &mut self,
        url: Url,
    ) -> Result<Vec<Bookmark<BookmarkId, UserId>>, sqlx::Error> {
        sqlx::query_as(
            r#"
               SELECT * FROM bookmarks WHERE url = ?;
            "#,
        )
        .bind(url.to_string())
        .fetch_all(&mut *self.txn)
        .await
    }

    /// Find the current user's bookmark with the given URL, if it exists.
    #[tracing::instrument(skip(self))]
    pub async fn find_bookmark_with_url(
        &mut self,
        url: &Url,
    ) -> Result<Option<Bookmark<BookmarkId, UserId>>, sqlx::Error> {
        sqlx::query_as(
            r#"
               SELECT * FROM bookmarks WHERE url = ? AND user_id = ?;
            "#,
        )
        .bind(url.to_string())
        .bind(self.user().id)
        .fetch_optional(&mut *self.txn)
        .await
    }

    /// Update the values on a bookmark for a user.
    ///
    ///
    /// ## Fields not modified
    /// - `id` and `user_id` - A user may only update the bookmarks belonging to them.
    ///
    /// - `accessed_at` and `created_at` - these timestamps can't be
    ///   manually reset.
    #[tracing::instrument(skip(self))]
    pub async fn update_bookmark(
        &mut self,
        bm: &Bookmark<BookmarkId, UserId>,
    ) -> Result<(), sqlx::Error> {
        let url = bm.url.as_str();
        sqlx::query!(
            r#"
              UPDATE bookmarks
              SET
                modified_at = datetime(),
                url = ?,
                title = ?,
                description = ?,
                website_title = ?,
                website_description = ?,
                unread = ?,
                shared = ?,
                notes = ?,
                import_properties = ?
              WHERE bookmark_id = ? AND user_id = ?
            "#,
            url,
            bm.title,
            bm.description,
            bm.website_title,
            bm.website_description,
            bm.unread,
            bm.shared,
            bm.notes,
            bm.import_properties,
            bm.id,
            bm.user_id,
        )
        .execute(&mut *self.txn)
        .await
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use anyhow::Context as _;
    use sqlx::SqlitePool;
    use url::Url;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_bookmark(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin_for_user("tester").await?;
        let to_add = Bookmark {
            id: (),
            user_id: (),
            created_at: Default::default(),
            modified_at: Some(Default::default()),
            accessed_at: Some(Default::default()),
            url: Url::parse("https://github.com/antifuchs/lz")?,
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
        let added = txn
            .add_bookmark(to_add.clone())
            .await
            .context("adding it")?;
        let retrieved = txn
            .get_bookmark_by_id(added.id())
            .await
            .context("retrieving it")?;

        assert_eq!(added, retrieved.id);
        assert_eq!(
            retrieved,
            Bookmark {
                id: added,
                user_id: txn.user().id,
                created_at: to_add.created_at,
                modified_at: to_add.modified_at,
                accessed_at: to_add.accessed_at,
                url: to_add.url.clone(),
                title: to_add.title,
                description: to_add.description,
                website_title: to_add.website_title,
                website_description: to_add.website_description,
                notes: to_add.notes,
                import_properties: to_add.import_properties,
                shared: to_add.shared,
                unread: to_add.unread,
            }
        );

        let retrieved_by_url = txn.find_bookmark_with_url(&to_add.url).await?;
        assert_eq!(Some(retrieved), retrieved_by_url);
        txn.commit().await?;
        Ok(())
    }
}
