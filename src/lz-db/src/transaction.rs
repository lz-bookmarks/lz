use std::marker::PhantomData;

use crate::Connection;

/// The mode that the transaction is in: Either read-write or read-only.
///
/// This is an optimization for reducing timeouts / lock contention on
/// readonly/read-write txns in a sqlite database, but should also
/// help with correctness and safety: A readonly transaction does not
/// have read-write methods defined.
pub trait TransactionMode {}

/// Read-only transaction mode. See [Transaction].
pub struct ReadOnly {}
impl TransactionMode for ReadOnly {}

/// Read-write transaction mode. See [Transaction].
pub struct ReadWrite {}
impl TransactionMode for ReadWrite {}

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
pub struct Transaction<M: TransactionMode = ReadWrite> {
    txn: sqlx::Transaction<'static, sqlx::sqlite::Sqlite>,
    user: User<UserId>,
    marker: PhantomData<M>,
}

/// An error that can occur when beginning a readonly transaction for a user.
#[derive(thiserror::Error, Debug)]
pub enum RoTransactionError {
    /// Any error raised by sqlx.
    #[error("sql datastore error")]
    Sqlx(#[from] sqlx::Error),

    /// User doesn't exist and can not be created due to the read-only
    /// nature of the connection.
    #[error("username {user} does not yet exist")]
    UserNotFound { user: String },
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
    pub async fn begin_for_user(
        &self,
        username: &str,
    ) -> Result<Transaction<ReadWrite>, sqlx::Error> {
        let mut txn = self.rw.begin().await?;
        let user = Connection::ensure_user(&mut txn, username).await?;
        Ok(Transaction {
            txn,
            user,
            marker: PhantomData,
        })
    }

    /// Begin a new read-only transaction as a given user.
    ///
    /// If the user with the given name doesn't exist yet, this raises
    /// an error that the database is opened in read-only mode.
    ///
    /// In order to commit the changes that happened in the
    /// transaction, call [`Transaction::commit`].
    pub async fn begin_ro_for_user(
        &self,
        username: &str,
    ) -> Result<Transaction<ReadOnly>, RoTransactionError> {
        let mut txn = if let Some(ro) = &self.ro {
            ro.begin()
        } else {
            self.rw.begin()
        }
        .await?;
        let user = Connection::get_user(&mut txn, username)
            .await?
            .ok_or_else(|| RoTransactionError::UserNotFound {
                user: username.to_string(),
            })?;
        Ok(Transaction {
            txn,
            user,
            marker: PhantomData,
        })
    }
}

impl Transaction<ReadWrite> {
    /// Commits the transaction.
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.txn.commit().await
    }
}

impl<M: TransactionMode> Transaction<M> {
    /// Return the user whom the transaction is concerning.
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

pub mod web;

pub(crate) mod criteria;
pub use criteria::{
    created_after_from_datetime, created_before_from_datetime, BookmarkSearch,
    BookmarkSearchCriteria, DateInput,
};

mod url;
pub use url::*;
