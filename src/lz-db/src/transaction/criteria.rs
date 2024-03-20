//! Search criteria translation in the DB

use std::fmt;

use sqlx::{query_builder::Separated, Sqlite};

use crate::{IdType, TagId, TagName, UserId};

/// A trait that allows translating a type internal to lz-db to a set of database query criteria.
pub trait BookmarkSearchCriteria {
    /// Adds a table to the query builder that the bookmarks query gets joined with.
    #[allow(unused_variables)]
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep
    }

    /// Inserts the data's criteria into a query WHERE clause, if applicable.
    #[allow(unused_variables)]
    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep
    }
}

/// A set of search criteria for a bookmark query.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkSearch {
    TagByName { name: TagName },
    TagById { id: TagId },
    User { id: UserId },
}

impl BookmarkSearchCriteria for BookmarkSearch {
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        match self {
            BookmarkSearch::TagByName { name } => name.bookmarks_join_table(sep),
            BookmarkSearch::TagById { id } => id.bookmarks_join_table(sep),
            BookmarkSearch::User { id } => id.bookmarks_join_table(sep),
        }
    }

    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        match self {
            BookmarkSearch::TagByName { name } => name.where_clause(sep),
            BookmarkSearch::TagById { id } => id.where_clause(sep),
            BookmarkSearch::User { id } => id.where_clause(sep),
        }
    }
}

/// Constricts a bookmark query to only return bookmarks that have the given tag name.
impl BookmarkSearchCriteria for TagName {
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        mut sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep.push(
            r#"SELECT bookmark_id FROM tags JOIN bookmark_tags USING (tag_id) WHERE tags.name ="#,
        );
        sep.push_bind_unseparated(self.0.to_string());
        sep
    }
}

/// Constricts a bookmark query to only return bookmarks having a tag with the given ID.
impl BookmarkSearchCriteria for TagId {
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        mut sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep.push(r#"SELECT bookmark_id FROM tags WHERE tags.id ="#);
        sep.push_bind_unseparated(self.id());
        sep
    }
}

/// Constricts a bookmark query to only return bookmarks that belong to the given user.
impl BookmarkSearchCriteria for UserId {
    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        mut sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep.push("user_id = ");
        sep.push_bind_unseparated(self.id());
        sep
    }
}
