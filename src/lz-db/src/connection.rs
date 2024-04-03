use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

/// A connection to an sqlite DB holding our bookmark data.
pub struct Connection {
    pub(crate) rw: sqlx::sqlite::SqlitePool,
    pub(crate) ro: Option<sqlx::sqlite::SqlitePool>,
}

impl Connection {
    /// Create a database connection to a file on disk.
    pub async fn from_path(path: &Path) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(&path)
            // Options from https://kerkour.com/sqlite-for-servers:
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .synchronous(SqliteSynchronous::Normal)
            .pragma("cache_size", "1000000000")
            .foreign_keys(true)
            .pragma("temp_store", "memory")
            // Some settings that just seem like a good idea:
            .shared_cache(true)
            .optimize_on_close(true, None);

        let rw = SqlitePool::connect_with(options.clone()).await?;
        let ro = Some(
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options.read_only(true))
                .await?,
        );
        Ok(Connection { rw, ro })
    }

    /// Create a database connection from an open SqlitePool.
    pub fn from_pool(rw: sqlx::sqlite::SqlitePool) -> Self {
        Self { rw, ro: None }
    }
}
