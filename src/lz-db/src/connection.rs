use sqlx::{sqlite::SqliteSynchronous, ConnectOptions as _, SqlitePool};
use std::str::FromStr as _;
use std::{path::Path, time::Duration};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

/// A connection to an sqlite DB holding our bookmark data.
pub struct Connection {
    pub(crate) db: sqlx::sqlite::SqlitePool,
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
        let pool = SqlitePool::connect_with(options).await?;
        Ok(Self::from_pool(pool))
    }

    /// Create a database connection from an open SqlitePool.
    pub fn from_pool(db: sqlx::sqlite::SqlitePool) -> Self {
        Self { db }
    }
}
