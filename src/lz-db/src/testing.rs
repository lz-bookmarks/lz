//! Helpers for testing an lz database

use std::marker::PhantomData;

use test_context::AsyncTestContext;
use tracing::level_filters::LevelFilter;

use crate::{Connection, Transaction, MIGRATOR};

/// Migrate the test database and bring it up to the latest revision
/// (this is the default).
pub struct DefaultMigrationBehavior();

/// Do not migrate the test database.
pub struct NoMigrationBehavior();

/// A test context, for use in lz-db integration tests
pub struct Context<M: MigrationBehavior = DefaultMigrationBehavior> {
    connection: Connection,
    phantom: PhantomData<M>,
}

impl<M: MigrationBehavior> Context<M> {
    pub const DEFAULT_USER: &'static str = "tester";

    /// Begins a transaction for the [default user][DEFAULT_USER].
    pub async fn begin(&self) -> Result<Transaction, sqlx::Error> {
        self.begin_for_user(Self::DEFAULT_USER).await
    }

    /// Begins a transaction for the named user.
    pub async fn begin_for_user(&self, name: &str) -> Result<Transaction, sqlx::Error> {
        self.connection.begin_for_user(name).await
    }

    /// Returns the SQLite DB pool used in this context
    pub fn db_pool(&mut self) -> &sqlx::SqlitePool {
        &self.connection.db
    }
}

/// A test context that doesn't run migrations on the database.
pub type NonMigratingContext = Context<NoMigrationBehavior>;

/// Sets up the default tracing subscriber and connects to a lz-db in-memory database pool.
///
/// By varying the `M` type argument on this context (it defaults to
/// [`DefaultMigrationBehavior`]), you can control whether database
/// migrations get applied to the database or not.
impl<M: MigrationBehavior> AsyncTestContext for Context<M> {
    async fn setup() -> Context<M> {
        let filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .with_env_var("LZ_TEST_RUST_LOG")
            .from_env_lossy();

        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(filter)
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::NEW
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            .with_target(false)
            .with_test_writer()
            .try_init();

        let mut pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("creating an in-memory sqlite pool");
        M::migrate(&mut pool)
            .await
            .expect("migrating the sqlite pool");
        let connection = Connection::from_pool(pool);
        Self {
            connection,
            phantom: Default::default(),
        }
    }
}

pub trait MigrationBehavior {
    fn migrate(
        pool: &mut sqlx::SqlitePool,
    ) -> impl std::future::Future<Output = sqlx::Result<()>> + Send;
}

/// Migrates the database fully, using the crate migrator.
impl MigrationBehavior for DefaultMigrationBehavior {
    async fn migrate(pool: &mut sqlx::SqlitePool) -> sqlx::Result<()> {
        MIGRATOR.run(&*pool).await?;
        Ok(())
    }
}

/// Does not migrate the database.
impl MigrationBehavior for NoMigrationBehavior {
    async fn migrate(_: &mut sqlx::SqlitePool) -> sqlx::Result<()> {
        Ok(())
    }
}
