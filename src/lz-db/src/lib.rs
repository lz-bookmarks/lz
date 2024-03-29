//! Database bindings and models for the `lz` bookmark manager

#[cfg(test)]
pub(crate) static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

mod connection;
mod transaction;

pub use connection::*;
pub use transaction::*;

#[cfg(test)]
pub mod testing;
#[cfg(test)]
pub use testing::{Context, NonMigratingContext};

#[cfg(test)]
mod tests {
    use crate::{NonMigratingContext, MIGRATOR};
    use anyhow::Result;
    use test_context::test_context;

    #[test_context(NonMigratingContext)]
    #[tokio::test]
    async fn apply_migrations(ctx: &mut NonMigratingContext) -> Result<()> {
        MIGRATOR.run(&*ctx.db_pool()).await?;
        Ok(())
    }
}
