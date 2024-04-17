use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use sqlx::query_scalar;
use url::Url;
use utoipa::{ToResponse, ToSchema};

use crate::{IdType, NoId, ReadWrite, Transaction, TransactionMode, UserId};

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
    delegate_display::DelegateDisplay,
)]
#[sqlx(transparent)]
#[serde(transparent)]
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
    NewBookmark = Bookmark<NoId, NoId>,
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
    #[serde(default)]
    pub notes: Option<String>,

    /// Whether the bookmark is "to read"
    #[serde(default)]
    pub unread: bool,

    /// Whether other users can see the bookmark.
    #[serde(default)]
    pub shared: bool,

    /// Properties imported from other systems.
    #[serde(skip_deserializing, skip_serializing)]
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

impl Bookmark<NoId, NoId> {
    /// Returns a Bookmark struct that has its IDs filled out.
    fn with_filled_ids(self, id: BookmarkId, user_id: UserId) -> Bookmark<BookmarkId, UserId> {
        let Bookmark {
            id: _empty_id,
            user_id: _empty_user_id,
            created_at,
            modified_at,
            accessed_at,
            url,
            title,
            description,
            website_title,
            website_description,
            notes,
            unread,
            shared,
            import_properties,
        } = self;
        Bookmark {
            id,
            user_id,
            created_at,
            modified_at,
            accessed_at,
            url,
            title,
            description,
            website_title,
            website_description,
            notes,
            unread,
            shared,
            import_properties,
        }
    }
}

/// # Working with Bookmarks
impl Transaction<ReadWrite> {
    /// Store a new bookmark in the database.
    #[tracing::instrument(skip(self))]
    pub async fn add_bookmark(
        &mut self,
        bm: Bookmark<NoId, NoId>,
    ) -> Result<Bookmark<BookmarkId, UserId>, sqlx::Error> {
        let user_id = self.user().id;
        let url_id = self.ensure_url(&bm.url).await?;
        let id = query_scalar!(
            r#"
              INSERT INTO bookmarks (
                user_id,
                created_at,
                modified_at,
                accessed_at,
                url_id,
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
            url_id,
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
        Ok(bm.with_filled_ids(BookmarkId(id), self.user().id))
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
    pub async fn update_bookmark<U: IdType<UserId> + std::fmt::Debug>(
        &mut self,
        bm: &Bookmark<BookmarkId, U>,
    ) -> Result<(), sqlx::Error> {
        let url_id = self.ensure_url(&bm.url).await?;
        let user_id = self.user().id;
        sqlx::query!(
            r#"
              UPDATE bookmarks
              SET
                modified_at = datetime(),
                url_id = ?,
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
            url_id,
            bm.title,
            bm.description,
            bm.website_title,
            bm.website_description,
            bm.unread,
            bm.shared,
            bm.notes,
            bm.import_properties,
            bm.id,
            user_id,
        )
        .execute(&mut *self.txn)
        .await
        .map(|_| ())
    }
}

/// Reading and finding [`Bookmark`]s
impl<M: TransactionMode> Transaction<M> {
    /// Retrieve the bookmark with the given ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_bookmark_by_id(
        &mut self,
        id: i64,
    ) -> Result<Bookmark<BookmarkId, UserId>, sqlx::Error> {
        sqlx::query_as(
            r#"
               SELECT *, urls.link AS url FROM bookmarks JOIN urls USING (url_id) WHERE bookmark_id = ? AND user_id = ?;
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
               SELECT *, urls.link AS url FROM bookmarks JOIN urls USING (url_id) WHERE url.link = ?;
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
               SELECT *, urls.link AS url FROM bookmarks JOIN urls USING (url_id) WHERE urls.link = ? AND user_id = ?;
            "#,
        )
            .bind(url.to_string())
            .bind(self.user().id)
            .fetch_optional(&mut *self.txn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use test_context::test_context;
    use testresult::TestResult;
    use url::Url;

    use crate::*;

    #[test_context(Context)]
    #[tokio::test]
    async fn roundtrip_bookmark(ctx: &mut Context) -> TestResult {
        let mut txn = ctx.begin().await?;
        let to_add = Bookmark {
            id: NoId,
            user_id: NoId,
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
        let added = txn.add_bookmark(to_add.clone()).await?;
        let retrieved = txn.get_bookmark_by_id(added.id.id()).await?;

        assert_eq!(added.id, retrieved.id);
        assert_eq!(retrieved, added);

        let retrieved_by_url = txn.find_bookmark_with_url(&to_add.url).await?;
        assert_eq!(Some(retrieved), retrieved_by_url);
        txn.commit().await?;
        Ok(())
    }
}
