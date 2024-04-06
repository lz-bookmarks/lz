use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::IdType;

/// The database ID of a tag.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, ToSchema, ToResponse)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(transparent))]
pub struct TagId(i64);

impl IdType<TagId> for TagId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A named tag, possibly assigned to multiple bookmarks.
///
/// See the section in [Transaction][Transaction#working-with-tags]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug, ToSchema, ToResponse)]
#[aliases(
    ExistingTag = Tag<TagId>,
)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Tag<ID: IdType<TagId>> {
    /// Database identifier of the tag.
    #[cfg_attr(feature = "server", sqlx(rename = "tag_id"))]
    pub id: ID,

    /// Name of the tag.
    pub name: String,

    /// When the tag was first created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Tag<TagId>> for TagId {
    fn from(val: Tag<TagId>) -> Self {
        val.id
    }
}

impl From<&Tag<TagId>> for TagId {
    fn from(val: &Tag<TagId>) -> Self {
        val.id
    }
}
