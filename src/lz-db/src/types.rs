//! Types that are understood by the lz data model.
//!
//! Watch out: This isn't an ORM, but the things that the lz-db
//! crate handles sure may feel like one at times.

/// A trait implemented by things that can be IDs.
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

mod user;
pub use user::*;

mod bookmark;
pub use bookmark::*;

mod import_properties;
pub use import_properties::*;

mod tag;
pub use tag::*;
