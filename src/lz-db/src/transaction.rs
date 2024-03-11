use crate::Connection;

/// A database transaction, operating on the behalf of an `lz` user.
///
/// Transactions are the main way that `lz` code uses the database:
/// This structure exposes operations on the DB, which allow working
/// with bookmarks, tags, and so on.
///
/// Once "done" with the unit of operation (be it a full import run or
/// an HTTP request), the transaction needs to be
/// [`commit`][Transaction::commit]ed.
#[derive(Debug)]
pub struct Transaction {
    txn: sqlx::Transaction<'static, sqlx::sqlite::Sqlite>,
    user: User<UserId>,
}

/// # Transactions
///
/// A transaction is the main way code interacts with the
/// database. All data-access methods are hanging off the transactions
/// created here.
impl Connection {
    /// Begin a new transaction as a given user.
    ///
    /// If the user with the given name doesn't exist yet, it is
    /// created inside that transaction.
    ///
    /// In order to commit the changes that happened in the
    /// transaction, call [`Transaction::commit`].
    pub async fn begin_for_user(&self, username: &str) -> Result<Transaction, sqlx::Error> {
        let mut txn = self.db.begin().await?;
        let user = Connection::ensure_user(&mut txn, username).await?;
        Ok(Transaction { txn, user })
    }
}

impl Transaction {
    /// Commits the transaction.
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.txn.commit().await
    }

    pub fn user(&self) -> &User<UserId> {
        &self.user
    }
}

pub trait IdType<T>: Copy {
    type Id;

    /// Returns the inner ID.
    fn id(self) -> Self::Id;
}

/// The "don't even think about it" type.
pub enum Never {}

/// The () type can be an ID for any DB type here.
///
/// This is useful for passing [`Bookmark`] to a creation function,
/// where we need no ID to be set.
impl<T> IdType<T> for () {
    type Id = Never;

    fn id(self) -> Self::Id {
        unreachable!("You mustn't try to access non-IDs.");
    }
}

mod bookmark;
pub use bookmark::*;

mod tag;
pub use tag::*;

mod user;
pub use user::*;

mod import_properties;
pub use import_properties::*;

pub mod cli;
pub mod web;
