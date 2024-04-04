//! # Stored URLs
//!
//!
//! We store links in a separate table from bookmarks, which allows
//! for multiple associations between a bookmark and a related link
//! ("discussed at", "referenced by")

use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
use sqlx::{prelude::*, types::Text};
use url::Url;
use utoipa::{ToResponse, ToSchema};

use crate::{BookmarkId, IdType, ReadWrite, Transaction, TransactionMode};

/// The database ID of a stored URL.
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
)]
#[sqlx(transparent)]
pub struct StoredUrlId(i64);

impl IdType<StoredUrlId> for StoredUrlId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, FromRow, ToSchema, ToResponse)]
#[aliases(
    ExistingUrl = StoredUrl<StoredUrlId>,
)]
pub struct StoredUrl<ID: IdType<StoredUrlId>> {
    /// Database identifier of the URL
    #[sqlx(rename = "url_id")]
    pub id: ID,

    /// The link that the URL points to
    pub link: Url,
}

/// # Reading stored URLs
impl<M: TransactionMode> Transaction<M> {
    /// Return a stored URL's ID if it exists in the database.
    pub async fn url_id_for_link(
        &mut self,
        link: &Url,
    ) -> Result<Option<StoredUrlId>, sqlx::Error> {
        let bm_url = Text(link);
        Ok(query_scalar!(
            r#"
              SELECT url_id FROM urls WHERE link = ?
            "#,
            bm_url,
        )
        .fetch_optional(&mut *self.txn)
        .await?
        .map(StoredUrlId))
    }
}

/// # Adding stored URLs
impl Transaction<ReadWrite> {
    /// Ensure a stored link exists in the database.
    #[tracing::instrument(skip(self))]
    pub async fn ensure_url(&mut self, link: &Url) -> Result<StoredUrlId, sqlx::Error> {
        if let Some(id) = self.url_id_for_link(&link).await? {
            return Ok(id);
        }
        let bm_url = Text(link);
        Ok(StoredUrlId(
            query_scalar!(
                r#"
                  INSERT INTO urls (link) VALUES (?)
                  RETURNING url_id
                "#,
                bm_url,
            )
            .fetch_one(&mut *self.txn)
            .await?,
        ))
    }

    /// Associate a bookmark with an additional link
    #[tracing::instrument(skip(self))]
    pub async fn associate_bookmark_link(
        &mut self,
        bm: &BookmarkId,
        associate: &StoredUrlId,
        context: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
              INSERT INTO bookmark_associations(
                bookmark_id,
                url_id,
                context
              ) VALUES (?, ?, ?)
            "#,
            bm,
            associate,
            context,
        )
        .execute(&mut *self.txn)
        .await?;
        Ok(())
    }
}
