use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::{ToResponse, ToSchema};

use crate::UserId;

use super::IdType;

/// The database ID of a bookmark.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, ToSchema, ToResponse)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(transparent))]
pub struct BookmarkId(pub(crate) i64);

impl IdType<BookmarkId> for BookmarkId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A bookmark saved by a user.
///
/// See the section in [Transaction][Transaction#working-with-bookmarks]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, ToSchema, ToResponse)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[aliases(
    ExistingBookmark = Bookmark<BookmarkId, UserId>,
)]
pub struct Bookmark<ID: IdType<BookmarkId>, UID: IdType<UserId>> {
    /// Database identifier of the bookmark
    #[cfg_attr(feature = "server", sqlx(rename = "bookmark_id"))]
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
    #[cfg_attr(feature = "server", sqlx(try_from = "&'a str"))]
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
    #[serde(skip_deserializing, skip_serializing)]
    #[cfg(feature = "server")]
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
