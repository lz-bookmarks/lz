use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::IdType;

/// The database ID of a user.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, ToSchema, ToResponse)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(transparent))]
pub struct UserId(pub(crate) i64);

impl IdType<UserId> for UserId {
    type Id = i64;

    fn id(self) -> Self::Id {
        self.0
    }
}

/// A user known the system.
///
/// The currently active user can be retrieved via
/// [`Transaction::user`].
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, ToSchema, ToResponse)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct User<ID: IdType<UserId>> {
    /// Database identifier of the user.
    #[cfg_attr(feature = "server", sqlx(rename = "user_id"))]
    pub id: ID,

    /// Name that the user authenticates as.
    pub name: String,

    /// Time that the user was created.
    ///
    /// This field is assigned in the database.
    pub created_at: chrono::DateTime<chrono::Utc>,
}
