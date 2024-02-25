

/// A connection to an sqlite DB holding our bookmark data.
pub struct Connection {
    pub(crate) db: sqlx::sqlite::SqlitePool,
}

impl Connection {
    /// Create a database connection from an open SqlitePool.
    pub fn from_pool(db: sqlx::sqlite::SqlitePool) -> Self {
        Self { db }
    }
}
