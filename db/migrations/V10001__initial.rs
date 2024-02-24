use barrel::{backend::Sqlite, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();
    m.create_table("bookmarks", |t| {
        t.add_column("bookmark_id", types::primary());
        t.add_column("url", types::text());
        t.add_column("title", types::text());
        t.add_column("description", types::text());
        t.add_column("website_title", types::text());
        t.add_column("notes", types::text());
    });

    m.create_table("tags", |t| {
        t.add_column("tag_id", types::primary());
        t.add_column("name", types::text());
    });

    m.create_table("bookmark_tags", |t| {
        t.add_column("bookmark_id", types::integer());
        t.add_column("tag_id", types::integer());
        t.set_primary_key(&["bookmark_id", "tag_id"]);
        t.add_foreign_key(&["bookmark_id"], "bookmarks", &["bookmark_id"]);
        t.add_foreign_key(&["tag_id"], "tags", &["tag_id"]);
    });

    m.make::<Sqlite>()
}
