//! Search criteria translation in the DB. See trait [`BookmarkSearchCriteria`].

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::query_builder::Separated;
use sqlx::Sqlite;
use utoipa::ToSchema;

use crate::{IdType, TagId, TagName, UserId};

/// A trait that allows translating a type internal to lz-db to a set
/// of database query criteria.
///
/// This is used by the
/// [`crate::Transaction::list_bookmarks_matching`] method in two
/// parts of the query assembly process. Trait implementors only need
/// to override the method that applies to the criteria you search
/// for; by default, each method is set to "do nothing" (i.e., not
/// modify the query and thus not insert anything into the query).
///
/// See [`sqlx::QueryBuilder::separated`] for details.
///
/// # Limiting the set of bookmark by their relationships
///
/// Relationships between bookmarks and tags are N:M, so in order to
/// find the bookmarks that have _all_ searched tags, we do an SQL
/// join with a set of queries / tables that return
/// `bookmark_id`s. Each of these queries is `INTERSECT`ed with the
/// others, resulting in an ever-narrower set of eligible bookmarks.
///
/// The method
/// [`bookmarks_join_table`](BookmarkSearchCriteria::bookmarks_join_table)
/// is responsible for putting these queries onto the query
/// builder.
///
/// The returned expressions must be a SELECT expression (not wrapped
/// in parens) that returns exactly one column, named `bookmark_id`.
///
/// # Limiting the set of bookmarks
///
/// As bookmarks themselves can have attributes that we'll want to
/// narrow on (URL text), dates, owning user, etc, these can be pretty
/// straightforwardly `AND`ed together, and that's exactly what we do
/// here.
///
/// The method [`where_clause`](BookmarkSearchCriteria::where_clause)
/// is responsible for adding any direct criteria expressions to the
/// query.
///
/// The expressions joined here are surrounded by parens so that
/// you don't have to worry about precedence of logic operators.
pub trait BookmarkSearchCriteria {
    /// Adds a table to the query builder that the bookmarks query gets joined with.
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep
    }

    /// Inserts the data's criteria into a query WHERE clause, if applicable.
    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum BookmarkSearchDatetimeField {
    Created,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum BookmarkSearchDatetimeOrientation {
    After,
    Before,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BookmarkSearchDateParams {
    date: DateInput,
    field: BookmarkSearchDatetimeField,
    orientation: BookmarkSearchDatetimeOrientation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DateInput(String);

impl FromStr for DateInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let naive = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", &s), "%Y-%m-%d %H:%M:%S");
        if naive.is_ok() {
            Ok(DateInput(s.to_string()))
        } else {
            Err(anyhow!("{} is not a valid YYYY-MM-DD date string", s))
        }
    }
}

/// The possible criteria that we can search for in a bookmark
/// query. See [BookmarkSearchCriteria].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum BookmarkSearch {
    /// Only list bookmarks that have matching timestamps
    #[serde(rename = "time")]
    ByDate { date: BookmarkSearchDateParams },

    /// Only list bookmarks that are tagged with a tag of the given name.
    #[serde(rename = "tag")]
    TagByName { tag: TagName },

    /// Only list bookmarks that are tagged with a tag with the given ID.
    TagById {
        #[serde(rename = "tag_id")]
        id: TagId,
    },

    /// Only list bookmarks belonging to the given user.
    User {
        #[serde(rename = "user_id")]
        id: UserId,
    },
}

impl BookmarkSearchCriteria for BookmarkSearch {
    fn bookmarks_join_table<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        match self {
            BookmarkSearch::ByDate { date } => date.bookmarks_join_table(sep),
            BookmarkSearch::TagByName { tag } => tag.bookmarks_join_table(sep),
            BookmarkSearch::TagById { id } => id.bookmarks_join_table(sep),
            BookmarkSearch::User { id } => id.bookmarks_join_table(sep),
        }
    }

    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        match self {
            BookmarkSearch::ByDate { date } => date.where_clause(sep),
            BookmarkSearch::TagByName { tag } => tag.where_clause(sep),
            BookmarkSearch::TagById { id } => id.where_clause(sep),
            BookmarkSearch::User { id } => id.where_clause(sep),
        }
    }
}

/// Constricts a bookmark query to only return bookmarks from before, after, or at a
/// datetime (with the field and orientation as parameters).
impl BookmarkSearchCriteria for BookmarkSearchDateParams {
    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        mut sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        let field = match self.field {
            BookmarkSearchDatetimeField::Created => "created_at",
        };
        let operand = match self.orientation {
            BookmarkSearchDatetimeOrientation::After => ">=",
            BookmarkSearchDatetimeOrientation::Before => "<=",
        };
        sep.push(format!("DATE({}, 'localtime') {} ", field, operand));
        sep.push_bind_unseparated(self.date.0.clone());
        sep
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

    fn where_clause<'qb, 'args, Sep: fmt::Display>(
        &self,
        mut sep: Separated<'qb, 'args, Sqlite, Sep>,
    ) -> Separated<'qb, 'args, Sqlite, Sep> {
        sep.push("1 = 1");
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

/// Convenience method to make a ByDate search object, tied to `created_at >=`.
pub fn created_after_from_datetime(date: DateInput) -> BookmarkSearch {
    BookmarkSearch::ByDate {
        date: BookmarkSearchDateParams {
            date,
            field: BookmarkSearchDatetimeField::Created,
            orientation: BookmarkSearchDatetimeOrientation::After,
        },
    }
}

/// Convenience method to make a ByDate search object, tied to `created_at <=`.
pub fn created_before_from_datetime(date: DateInput) -> BookmarkSearch {
    BookmarkSearch::ByDate {
        date: BookmarkSearchDateParams {
            date,
            field: BookmarkSearchDatetimeField::Created,
            orientation: BookmarkSearchDatetimeOrientation::Before,
        },
    }
}
