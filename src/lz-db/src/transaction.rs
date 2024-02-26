use crate::Connection;

/// A database transaction
#[derive(Debug)]
pub struct Transaction<'c> {
    txn: sqlx::Transaction<'c, sqlx::sqlite::Sqlite>,
}

/// # Transactions
///
/// A transaction is the main way code interacts with the
/// database. All data-access methods are hanging off the transactions
/// created here.
impl Connection {
    /// Begin a new transaction and return it.
    ///
    /// In order to commit the changes that happened in the
    /// transaction, call [`Transaction::commit`].
    pub async fn begin<'c>(&'c self) -> Result<Transaction<'c>, sqlx::Error> {
        Ok(Transaction {
            txn: self.db.begin().await?,
        })
    }
}

impl<'c> Transaction<'c> {
    /// Commits the transaction.
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.txn.commit().await
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
