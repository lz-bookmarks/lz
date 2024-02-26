//! The `User` model.
//!
//! Note that lz doesn't support authentication or authorization as
//! such: If you have a thing that authenticates users, lz will take
//! that authentication data and identify the person accessing bookmarks
//! that way. But it will not attempt to do access control.

use crate::IdType;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query_as};

/// The database ID of a user.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, sqlx::Type)]
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
/// See the section in [Transaction][Transaction#working-with-users]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, FromRow)]
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
    #[tracing::instrument()]
    pub(crate) async fn ensure_user<'c>(
        txn: &mut sqlx::Transaction<'c, sqlx::Sqlite>,
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

#[cfg(test)]
mod tests {
    use crate::*;
    use sqlx::SqlitePool;

    #[test_log::test(sqlx::test(migrator = "MIGRATOR"))]
    fn roundtrip_user(pool: SqlitePool) -> anyhow::Result<()> {
        let conn = Connection::from_pool(pool);
        let txn = conn.begin_for_user("tester").await?;
        assert_eq!(txn.user().name, "tester");
        Ok(())
    }
}
