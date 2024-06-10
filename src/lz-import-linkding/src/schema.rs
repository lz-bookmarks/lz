//! Linkding's DB schema

use std::collections::HashMap;

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

impl<'c> LinkdingTransaction<'c> {
    pub(crate) fn all_tags(&mut self) -> BoxStream<Result<Tag, sqlx::Error>> {
        sqlx::query_as(r#"SELECT * FROM bookmarks_tag ORDER BY date_added"#).fetch(&mut *self.txn)
    }

    pub(crate) fn all_bookmarks(&mut self) -> BoxStream<Result<Bookmark, sqlx::Error>> {
        sqlx::query_as(r#"SELECT * FROM bookmarks_bookmark ORDER BY date_added, id"#)
            .fetch(&mut *self.txn)
    }

    pub(crate) fn all_taggings(&mut self) -> BoxStream<Result<BookmarkTag, sqlx::Error>> {
        sqlx::query_as(r#"SELECT * FROM bookmarks_bookmark_tags ORDER BY bookmark_id, id"#)
            .fetch(&mut *self.txn)
    }
}

/// An optional URL.
///
/// This newtype exists because sqlx can not directly attempt
/// conversion from &str to Url.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct OptionalUrl(Option<Url>);

impl<'a> TryFrom<&'a str> for OptionalUrl {
    type Error = <Url as TryFrom<&'a str>>::Error;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(OptionalUrl(None))
        } else {
            Ok(OptionalUrl(Some(Url::try_from(value)?)))
        }
    }
}

/// A bookmark that lives in Linkding.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct Bookmark {
    /// Database identifier of the bookmark
    pub id: i64,
    pub owner_id: i64,

    pub date_added: chrono::DateTime<chrono::Utc>,
    pub date_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub date_accessed: Option<chrono::DateTime<chrono::Utc>>,

    #[sqlx(try_from = "&'a str")]
    pub url: Url,
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub website_title: Option<String>,
    pub website_description: Option<String>,
    pub unread: bool,

    #[sqlx(try_from = "&'a str")]
    pub web_archive_snapshot_url: OptionalUrl,
    pub favicon_file: String,

    pub is_archived: bool,
    pub shared: bool,
}

impl Bookmark {
    pub fn as_lz_bookmark(&self) -> lz_db::Bookmark<lz_db::NoId, lz_db::NoId> {
        let url = self.url.clone();
        let mut other = lz_db::Bookmark {
            id: lz_db::NoId,
            user_id: lz_db::NoId,
            url,
            created_at: Default::default(),
            modified_at: Default::default(),
            accessed_at: Default::default(),
            title: Default::default(),
            description: Default::default(),
            website_title: Default::default(),
            website_description: Default::default(),
            notes: Default::default(),
            unread: Default::default(),
            shared: Default::default(),
            import_properties: Default::default(),
        };
        self.overwrite_into_lz_bookmark(&mut other);
        other
    }

    pub fn overwrite_into_lz_bookmark<
        ID: lz_db::IdType<lz_db::BookmarkId>,
        UID: lz_db::IdType<lz_db::UserId>,
    >(
        &self,
        other: &mut lz_db::Bookmark<ID, UID>,
    ) {
        let existing_import_properties = other.import_properties.clone();

        let import_properties = existing_import_properties.map_or_else(
            || {
                Some(sqlx::types::Json(lz_db::ImportProperties {
                    by_system: {
                        let mut by_system = HashMap::new();
                        by_system.insert(
                            lz_db::ImportableSystem::Linkding,
                            serde_json::to_value(self)
                                .expect("cleanly convert to serde_json::Value"),
                        );
                        by_system
                    },
                }))
            },
            |mut existing| {
                existing.0.by_system.insert(
                    lz_db::ImportableSystem::Linkding,
                    serde_json::to_value(self).expect("cleanly convert to serde_json::Value"),
                );
                Some(existing)
            },
        );

        // Sometime in 2023, linkding started saving an empty
        // description if none was set by the user. Let's undo this,
        // it's annoying:
        let title = match (self.title.as_str(), self.website_title.as_deref()) {
            ("", wt) => wt.map(String::from).unwrap_or_default().clone(),
            (title, _) => title.to_string(),
        };
        let description = match (
            self.description.as_deref(),
            self.website_description.as_deref(),
        ) {
            (Some("") | None, Some(wd)) => Some(wd.to_string().clone()),
            (description, _) => description.map(|d| d.to_string()),
        };

        other.created_at = self.date_added;
        other.modified_at = self.date_modified;
        other.accessed_at = self.date_accessed;
        other.url = self.url.clone();
        other.title = title;
        other.description = description;
        other.website_title.clone_from(&self.website_title);
        other
            .website_description
            .clone_from(&self.website_description);
        other.notes = match self.notes.as_ref() {
            None => None,
            Some(n) if n.is_empty() => None,
            Some(n) => Some(n.to_string()),
        };
        other.unread = self.unread;
        other.shared = self.shared;
        other.import_properties = import_properties;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct Tag {
    pub id: i64,
    pub name: String,
    pub date_added: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, FromRow)]
pub(crate) struct BookmarkTag {
    pub id: i64,
    pub bookmark_id: i64,
    pub tag_id: i64,
}
