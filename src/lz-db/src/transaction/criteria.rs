//! Search criteria translation in the DB

use sqlx::{QueryBuilder, Sqlite};

use crate::{IdType, TagId, TagName, UserId};

/// A trait that allows translating a type internal to lz-db to a set of database query criteria.
pub trait BookmarkSearchCriteria<'args> {
    /// Adds a table to the query builder that the bookmarks query gets joined with.
    ///
    /// The query must return `bookmark_id` field, and when adding it
    /// to the query builder, must insert a preceding `INTERSECT` if
    /// `is_first_clause` is true.
    #[allow(unused_variables)]
    fn bookmarks_join_table(
        &self,
        qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        qb
    }

    /// Inserts the data's criteria into a query WHERE clause, if applicable.
    ///
    /// It's expected that implementations of this insert an AND
    /// operator, if `is_first_clause` is true.
    #[allow(unused_variables)]
    fn where_clause(
        &self,
        qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        qb
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

impl<'args> BookmarkSearchCriteria<'args> for BookmarkSearch {
    fn bookmarks_join_table(
        &self,
        qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        match self {
            BookmarkSearch::TagByName { name } => name.bookmarks_join_table(qb, is_first_clause),
            BookmarkSearch::TagById { id } => id.bookmarks_join_table(qb, is_first_clause),
            BookmarkSearch::User { id } => id.bookmarks_join_table(qb, is_first_clause),
        }
    }

    fn where_clause(
        &self,
        qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        match self {
            BookmarkSearch::TagByName { name } => name.where_clause(qb, is_first_clause),
            BookmarkSearch::TagById { id } => id.where_clause(qb, is_first_clause),
            BookmarkSearch::User { id } => id.where_clause(qb, is_first_clause),
        }
    }
}

/// Constricts a bookmark query to only return bookmarks that have the given tag name.
impl<'args> BookmarkSearchCriteria<'args> for TagName {
    fn bookmarks_join_table(
        &self,
        mut qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        if !is_first_clause {
            qb.push(" INTERSECT ");
        }
        qb.push(
            r#"SELECT bookmark_id FROM tags JOIN bookmark_tags USING (tag_id) WHERE tags.name ="#,
        );
        qb.push_bind(self.0.to_string());
        qb
    }
}

/// Constricts a bookmark query to only return bookmarks having a tag with the given ID.
impl<'args> BookmarkSearchCriteria<'args> for TagId {
    fn bookmarks_join_table(
        &self,
        mut qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        if !is_first_clause {
            qb.push(" INTERSECT ");
        }
        qb.push(r#"SELECT bookmark_id FROM tags WHERE tags.id ="#);
        qb.push_bind(self.id());
        qb
    }
}

/// Constricts a bookmark query to only return bookmarks that belong to the given user.
impl<'args> BookmarkSearchCriteria<'args> for UserId {
    fn where_clause(
        &self,
        mut qb: QueryBuilder<'args, Sqlite>,
        is_first_clause: bool,
    ) -> QueryBuilder<'args, Sqlite> {
        if !is_first_clause {
            qb.push(" AND ");
        }
        qb.push("user_id = ");
        qb.push_bind(self.id());
        qb
    }
}
