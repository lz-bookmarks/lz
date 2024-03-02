//! Linkding's DB schema

use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use url::Url;

pub struct LinkdingTransaction<'c> {
    txn: sqlx::Transaction<'c, sqlx::sqlite::Sqlite>,
}

impl<'c> LinkdingTransaction<'c> {
    pub async fn from_pool(db: &mut sqlx::sqlite::SqlitePool) -> Result<Self, sqlx::Error> {
        let txn = db.begin().await?;
        Ok(Self { txn })
    }
}

/// A bookmark that lives in Linkding.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct Bookmark {
    /// Database identifier of the bookmark
    pub id: i64,
    pub owner_id: i64,

    pub date_added: chrono::DateTime<chrono::Utc>,
    pub date_modified: chrono::DateTime<chrono::Utc>,
    pub date_accessed: chrono::DateTime<chrono::Utc>,

    #[sqlx(try_from = "&'a str")]
    pub url: Url,
    pub title: String,
    pub description: String,
    pub website_title: Option<String>,
    pub website_description: Option<String>,
    pub unread: bool,

    #[sqlx(try_from = "&'a str")]
    pub web_archive_snapshot_url: Url,
    pub favicon_file: String,

    pub is_archived: bool,
    pub shared: bool,
    pub notes: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct Tag {
    pub id: i64,
    pub name: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
}

impl<'c> LinkdingTransaction<'c> {
    pub(crate) fn all_tags(&mut self) -> BoxStream<Result<Tag, sqlx::Error>> {
        sqlx::query_as(r#"SELECT * FROM bookmarks_tag"#).fetch(&mut *self.txn)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct BookmarkTag {
    pub id: i64,
    pub bookmark_id: i64,
    pub tag_id: i64,
}
