//! Database bindings and models for the `lz` bookmark manager

#[cfg(test)]
pub(crate) static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

mod connection;
mod transaction;

pub use connection::*;
pub use transaction::*;

#[cfg(test)]
mod tests {
    use crate::MIGRATOR;
    use anyhow::Result;
    use sqlx::sqlite::SqlitePool;

    #[test_log::test(sqlx::test(migrations = false))]
    async fn apply_migrations(pool: SqlitePool) -> Result<()> {
        MIGRATOR.run(&pool).await?;
        Ok(())
    }
}
