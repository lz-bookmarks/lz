//! Routines that translate Linkding's models into ours.

use std::collections::HashMap;

use anyhow::Context as _;
use futures::{stream::StreamExt as _, TryStreamExt as _};

use crate::{
    schema::{self, LinkdingTransaction},
    DuplicateBehavior,
};

pub struct Migration<'c> {
    db: lz_db::Transaction<'c>,
    linkding_tx: LinkdingTransaction<'c>,
    tag_ids: HashMap<i64, lz_db::TagId>,
    bookmark_ids: HashMap<i64, lz_db::BookmarkId>,
    on_duplicate: DuplicateBehavior,
}

impl<'c> Migration<'c> {
    pub fn new(
        db: lz_db::Transaction<'c>,
        linkding_tx: LinkdingTransaction<'c>,
        on_duplicate: DuplicateBehavior,
    ) -> Self {
        Self {
            db,
            linkding_tx,
            on_duplicate,
            tag_ids: Default::default(),
            bookmark_ids: Default::default(),
        }
    }

    /// Run the migration from a linkding export into lz.
    ///
    /// This does the following:
    /// 1. Creates all tags that exist in the linkding DB
    /// 2. Creates all bookmarks (skipping or overwriting duplicates)
    /// 3. Tags all the linkding bookmarks with tags from linkding.
    ///
    /// All created bookmarks get their import_properties.by_system
    /// JSON object filled with a `linkding` property, containing all
    /// the fields we recognized.
    #[tracing::instrument(skip(self))]
    pub async fn migrate(mut self) -> anyhow::Result<()> {
        self.translate_tags().await.context("translating tags")?;
        tracing::info!(tag_count = self.tag_ids.iter().len(), "Tags imported");

        self.translate_bookmarks()
            .await
            .context("translating bookmarks")?;
        tracing::info!(
            bookmark_count = self.bookmark_ids.iter().len(),
            "Bookmarks imported"
        );
        self.db.commit().await?;
        Ok(())
    }

    /// Translate all linkding tags into lz tags (creating them if necessary).
    #[tracing::instrument(skip(self))]
    async fn translate_tags(&mut self) -> Result<(), sqlx::Error> {
        let tag_stream = self.linkding_tx.all_tags();
        let mut chunks = tag_stream.chunks(1024);
        while let Some(maybe_tags) = chunks.next().await {
            let tags = maybe_tags
                .into_iter()
                .collect::<Result<Vec<schema::Tag>, sqlx::Error>>()?;

            let lz_side_tags = self.db.ensure_tags(tags.iter().map(|t| &t.name)).await?;
            let li_tags: HashMap<&str, i64> =
                tags.iter().map(|t| (t.name.as_str(), t.id)).collect();
            for tag in lz_side_tags {
                let li_tag_id = li_tags
                    .get(tag.name.as_str())
                    .expect("all tags to have corresponding names");
                self.tag_ids.insert(*li_tag_id, tag.id);
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn translate_bookmarks(&mut self) -> Result<(), sqlx::Error> {
        let mut bookmark_stream = self.linkding_tx.all_bookmarks();
        while let Some(bookmark) = bookmark_stream.try_next().await? {
            let url = bookmark.url.clone();
            if let Some(mut existing) = self.db.find_bookmark_with_url(&url).await? {
                match self.on_duplicate {
                    DuplicateBehavior::Skip => {
                        tracing::warn!(url=%bookmark.url, existing_created_at=?existing.created_at, "Skipping dupliate bookmark");
                    }
                    DuplicateBehavior::Overwrite => {
                        bookmark.overwrite_into_lz_bookmark(&mut existing);
                        self.db.update_bookmark(&existing).await?;
                        self.bookmark_ids.insert(bookmark.id, existing.id);
                        tracing::debug!(url=%bookmark.url, id=?existing.id, "overwrote bookmark");
                    }
                }
                continue;
            }
            let to_add = bookmark.as_lz_bookmark();
            let added_id = self.db.add_bookmark(to_add.clone()).await.map_err(|e| {
                tracing::error!(url=%bookmark.url, error=%e, error_debug=?e, "Could not add bookmark");
                e
            })?;
            self.bookmark_ids.insert(bookmark.id, added_id);
            tracing::debug!(url=%to_add.url, ?added_id, "added bookmark");
        }
        Ok(())
    }
}