use crate::Connection;

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use sqlx::query_scalar;
use sqlx::types::Text;

use url::Url;

/// A bookmark saved by a user.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow)]
pub struct Bookmark {
    /// URL that the bookmark points to.
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

/// A named tag, possibly assigned to multiple bookmarks.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Tag {
    /// Database identifier of the tag.
    pub tag_id: u64,

    /// Name of the tag.
    pub name: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct BookmarkId(i64);

impl Connection {
    pub async fn add_bookmark(&mut self, bm: Bookmark) -> Result<BookmarkId, sqlx::Error> {
        let mut txn = self.db.begin().await?; // TODO: take an existing txn
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
        .fetch_one(&mut *txn)
        .await?;
        txn.commit().await?;
        Ok(BookmarkId(id))
    }

    // /// Retrieve the bookmark with the given ID.
    // ///
    // /// This does not retrieve the tags on the bookmark, nor anything
    // /// else tied to it. Just the bookmark object.
    // pub fn get_bookmark_by_id(
    //     &mut self,
    //     id: i64,
    // ) -> Result<DatabaseEntry<Bookmark>, rusqlite::Error> {
    //     self.sqlite.query_row(
    //         indoc! {
    //             r#"
    //               SELECT * FROM bookmarks WHERE bookmark_id = :id;
    //             "#,
    //         },
    //         rusqlite::named_params! { ":id": id },
    //         |row| {
    //             Ok(DatabaseEntry {
    //                 id: row.get("bookmark_id")?,
    //                 obj: Bookmark {
    //                     url: row.get("url")?,
    //                     title: row.get("title")?,
    //                     description: row.get("description")?,
    //                     website_title: row.get("website_title")?,
    //                     website_description: row.get("website_description")?,
    //                     notes: row.get("notes")?,
    //                 },
    //             })
    //         },
    //     )
    // }
}

// #[cfg(test)]
// mod tests {
//     use test_log::test;
//     use url::Url;

//     use super::*;

//     #[test]
//     fn roundtrip_bookmark() -> anyhow::Result<()> {
//         let mut conn = Connection::open_in_memory()?.ensure_migrated()?;
//         let tags = ["code".to_string(), "high_quality".to_string()]
//             .into_iter()
//             .collect();
//         let to_add = Bookmark {
//             url: Url::parse("https://github.com/antifuchs/lz")?,
//             title: "The lz repo".to_string(),
//             description: "This is a great repo with excellent code.".to_string(),
//             website_title: Some("lz, the bookmarks manager".to_string()),
//             website_description: Some(
//                 "Please do not believe in the quality of this code.".to_string(),
//             ),
//             notes: "No need to run tests.".to_string(),
//         };
//         let added = conn.add_bookmark(to_add, tags)?;

//         let stored = conn.get_bookmark_by_id(added.id)?;
//         assert_eq!(added, stored);
//         Ok(())
//     }
// }
