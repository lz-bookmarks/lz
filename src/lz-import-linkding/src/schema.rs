//! Linkding's DB schema

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use url::Url;

/// A bookmark that lives in Linkding.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct Bookmark {
    /// Database identifier of the bookmark
    pub id: i64,

    /// ID of the user who owns the bookmark
    pub owner_id: i64,

    pub date_added: chrono::DateTime<chrono::Utc>,
    pub date_modified: chrono::DateTime<chrono::Utc>,
    pub date_accessed: chrono::DateTime<chrono::Utc>,

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

    pub unread: bool,

    #[sqlx(try_from = "&'a str")]
    pub web_archive_snapshot_url: Url,

    pub shared: bool,

    pub favicon_file: String,

    /// Private notes that the user attached to the bookmark.
    pub notes: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow)]
pub struct Tag {
    /// Database identifier of the tag.
    #[sqlx(rename = "tag_id")]
    pub id: i64,

    /// Name of the tag.
    pub name: String,

    pub date_added: chrono::DateTime<chrono::Utc>,
}
