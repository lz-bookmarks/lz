//! Routines that translate Linkding's models into ours.

use std::collections::HashMap;

use futures::{
    stream::{BoxStream, StreamExt},
    TryStreamExt as _,
};

use crate::schema::{self, LinkdingTransaction};

#[tracing::instrument(skip(linkding_tx, db))]
pub async fn migrate<'c>(
    mut linkding_tx: LinkdingTransaction<'c>,
    mut db: &mut lz_db::Transaction<'c>,
) -> anyhow::Result<()> {
    let tag_translation = translate_tags(&mut db, linkding_tx.all_tags()).await?;
    tracing::info!(tag_count = tag_translation.iter().len(), "Tags done");

    let bm_translation = translate_bookmarks(&mut db, linkding_tx.all_bookmarks()).await?;
    Ok(())
}

/// Translate all linkding tags into lz tags (creating them if necessary).
#[tracing::instrument(skip(db, tag_stream))]
async fn translate_tags<'c, 's>(
    db: &mut lz_db::Transaction<'c>,
    tag_stream: BoxStream<'s, Result<schema::Tag, sqlx::Error>>,
) -> Result<HashMap<i64, lz_db::TagId>, sqlx::Error>
where
    'c: 's,
{
    let mut translated = HashMap::new();
    let mut chunks = tag_stream.chunks(1024);
    while let Some(maybe_tags) = chunks.next().await {
        let tags = maybe_tags
            .into_iter()
            .collect::<Result<Vec<schema::Tag>, sqlx::Error>>()?;

        let lz_side_tags = db.ensure_tags(tags.iter().map(|t| &t.name)).await?;
        let li_tags: HashMap<&str, i64> = tags.iter().map(|t| (t.name.as_str(), t.id)).collect();
        for tag in lz_side_tags {
            let li_tag_id = li_tags
                .get(tag.name.as_str())
                .expect("all tags to have corresponding names");
            translated.insert(*li_tag_id, tag.id);
        }
    }
    Ok(translated)
}

#[tracing::instrument(skip(db, bookmark_stream))]
async fn translate_bookmarks<'c, 's>(
    db: &mut lz_db::Transaction<'c>,
    mut bookmark_stream: BoxStream<'s, Result<schema::Bookmark, sqlx::Error>>,
) -> Result<HashMap<i64, lz_db::BookmarkId>, sqlx::Error> {
    let mut translated = HashMap::new();
    while let Some(bookmark) = bookmark_stream.try_next().await? {
        let added_id = db.add_bookmark(bookmark.as_lz_bookmark()).await?;
        translated.insert(bookmark.id, added_id);
        tracing::info!(?bookmark, ?added_id, "added bookmark");
    }
    Ok(translated)
}
