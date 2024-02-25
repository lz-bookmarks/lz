use std::path::Path;

/// A connection to an sqlite DB holding our bookmark data.
pub struct Connection {
    sqlite: rusqlite::Connection,
}

impl From<rusqlite::Connection> for Connection {
    fn from(sqlite: rusqlite::Connection) -> Self {
        Connection { sqlite }
    }
}

impl Connection {
    /// Open a connection to a sqlite3 database file.
    ///
    /// If the file doesn't exist yet, it is created. In order to
    /// become usable, the migrations on it need to be run.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<NonLiveConnection, rusqlite::Error> {
        Ok(NonLiveConnection {
            sqlite: rusqlite::Connection::open(path)?,
        })
    }
}

/// A connection that may or may not be up to date.
///
/// Use [`ensure_migrated`] to get a usable [`Connection`].
pub struct NonLiveConnection {
    sqlite: rusqlite::Connection,
}

impl NonLiveConnection {
    pub fn ensure_migrated(self) -> Result<Connection, refinery::Error> {
        let mut sqlite = self.sqlite;
        crate::run_migrations(&mut sqlite)?;
        Ok(Connection { sqlite })
    }
}
