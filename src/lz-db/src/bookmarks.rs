use std::{collections::BTreeSet, rc::Rc};

use crate::Connection;
use indoc::indoc;
use lz_core::Bookmark;
use rusqlite::{named_params, types::Value};
use serde::Serialize;
use tracing::info;

/// An entry in a table, as returned by the database.
#[derive(Serialize, PartialEq, Eq, Debug)]
pub struct DatabaseEntry<T: Serialize + PartialEq + Eq> {
    pub id: i64,
    pub obj: T,
}

impl Connection {
    pub fn add_bookmark(
        &mut self,
        bm: Bookmark,
        tags: BTreeSet<String>,
    ) -> Result<DatabaseEntry<Bookmark>, rusqlite::Error> {
        let txn = self.sqlite.transaction()?;
        let mut query_existing = txn.prepare(indoc! {r#"
              SELECT tag_id FROM tags WHERE name IN rarray(:names)
            "#})?;
        let existing_q_tags = Rc::new(
            tags.iter()
                .cloned()
                .map(Value::from)
                .collect::<Vec<Value>>(),
        );
        let existing_tags = query_existing
            .query_map(
                named_params! {
                    ":names": existing_q_tags
                },
                |row| row.get::<usize, i64>(0),
            )?
            .collect::<Result<Vec<i64>, rusqlite::Error>>()?;
        info!(?existing_tags);
        drop(query_existing);
        txn.execute(
            indoc! {r#"
              INSERT INTO bookmarks (
                url,
                title,
                description,
                website_title,
                website_description,
                notes
              ) VALUES (
                :url,
                :title,
                :description,
                :website_title,
                :website_description,
                :notes
              );
            "#},
            rusqlite::named_params! {
                ":url": bm.url.to_string(),
                ":title": bm.title,
                ":description": bm.description,
                ":website_title": bm.website_title,
                ":website_description": bm.website_description,
                ":notes": bm.notes,
            },
        )?;
        let entry = DatabaseEntry {
            id: txn.last_insert_rowid(),
            obj: bm,
        };
        txn.commit()?;
        Ok(entry)
    }

    /// Retrieve the bookmark with the given ID.
    ///
    /// This does not retrieve the tags on the bookmark, nor anything
    /// else tied to it. Just the bookmark object.
    pub fn get_bookmark_by_id(
        &mut self,
        id: i64,
    ) -> Result<DatabaseEntry<Bookmark>, rusqlite::Error> {
        self.sqlite.query_row(
            indoc! {
                r#"
                  SELECT * FROM bookmarks WHERE bookmark_id = :id;
                "#,
            },
            rusqlite::named_params! { ":id": id },
            |row| {
                Ok(DatabaseEntry {
                    id: row.get("bookmark_id")?,
                    obj: Bookmark {
                        url: row.get("url")?,
                        title: row.get("title")?,
                        description: row.get("description")?,
                        website_title: row.get("website_title")?,
                        website_description: row.get("website_description")?,
                        notes: row.get("notes")?,
                    },
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;
    use url::Url;

    use super::*;

    #[test]
    fn roundtrip_bookmark() -> anyhow::Result<()> {
        let mut conn = Connection::open_in_memory()?.ensure_migrated()?;
        let tags = ["code".to_string(), "high_quality".to_string()]
            .into_iter()
            .collect();
        let to_add = Bookmark {
            url: Url::parse("https://github.com/antifuchs/lz")?,
            title: "The lz repo".to_string(),
            description: "This is a great repo with excellent code.".to_string(),
            website_title: Some("lz, the bookmarks manager".to_string()),
            website_description: Some(
                "Please do not believe in the quality of this code.".to_string(),
            ),
            notes: "No need to run tests.".to_string(),
        };
        let added = conn.add_bookmark(to_add, tags)?;

        let stored = conn.get_bookmark_by_id(added.id)?;
        assert_eq!(added, stored);
        Ok(())
    }
}
