//! The `User` model.
//!
//! Note that lz doesn't support authentication or authorization as
//! such: If you have a thing that authenticates users, lz will take
//! that authentication data and identify the person accessing bookmarks
//! that way. But it will not attempt to do access control.

use crate::{IdType, Transaction};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query_as};
use utoipa::{ToResponse, ToSchema};

/// The database ID of a user.
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
pub struct UserId(i64);

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
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow, ToSchema, ToResponse)]
pub struct User<ID: IdType<UserId>> {
    /// Database identifier of the user.
    #[sqlx(rename = "user_id")]
    pub id: ID,

    /// Name that the user authenticates as.
    pub name: String,

    /// Time that the user was created.
    ///
    /// This field is assigned in the database.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl crate::Connection {
    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), ret(level = tracing::Level::DEBUG), skip(txn))]
    pub(crate) async fn ensure_user(
        txn: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        name: &str,
    ) -> Result<User<UserId>, sqlx::Error> {
        if let Some(user) = query_as(r#"SELECT * FROM users WHERE name = ?"#)
            .bind(name)
            .fetch_optional(&mut **txn)
            .await?
        {
            return Ok(user);
        }

        query_as(
            r#"
              INSERT INTO users (
                name,
                created_at
              ) VALUES (
                ?,
                datetime()
              )
              RETURNING *
            "#,
        )
        .bind(name)
        .fetch_one(&mut **txn)
        .await
    }
}

/// # Working with [`User`]s
impl Transaction {
    /// Retrieve a user with a given name.
    #[tracing::instrument(err(Debug, level = tracing::Level::WARN), ret, skip(self))]
    pub async fn get_user_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<User<UserId>>, sqlx::Error> {
        query_as(r#"SELECT * FROM users WHERE name = ?"#)
            .bind(name)
            .fetch_optional(&mut *self.txn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use sqlx::SqlitePool;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_user(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let mut txn = conn.begin_for_user("tester").await?;
        assert_eq!(txn.user().name, "tester");

        let user = txn.get_user_by_name("tester").await?;
        assert_eq!(Some(txn.user()), user.as_ref());
        Ok(())
    }
}
