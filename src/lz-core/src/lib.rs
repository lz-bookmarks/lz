//! Base data types for bookmarking models and such.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use url::Url;

/// A bookmark saved by a user.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Bookmark {
    /// URL that the bookmark points to.
    pub url: Url,

    /// Title that the user gave the bookmark.
    pub title: String,

    /// Description of the bookmark, possibly extracted from the website.
    pub description: String,

    /// Original title extracted from the website.
    pub website_title: Option<String>,

    /// Original description extracted from the website.
    pub website_description: Option<String>,

    /// Private notes that the user attached to the bookmark.
    pub notes: String,
}

/// A named tag, possibly assigned to multiple bookmarks.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Tag {
    /// Database identifier of the tag.
    pub tag_id: u64,

    /// Name of the tag.
    pub name: String,
}
