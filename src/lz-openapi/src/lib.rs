#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    use serde::{Deserialize, Serialize};
    #[allow(unused_imports)]
    use std::convert::TryFrom;
    /// Error types.
    pub mod error {
        /// Error from a TryFrom or FromStr implementation.
        pub struct ConversionError(std::borrow::Cow<'static, str>);
        impl std::error::Error for ConversionError {}
        impl std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///A bookmark, including tags and associations on it.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A bookmark, including tags and associations on it.",
    ///  "type": "object",
    ///  "required": [
    ///    "associations",
    ///    "bookmark",
    ///    "tags"
    ///  ],
    ///  "properties": {
    ///    "associations": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AssociatedLink"
    ///      }
    ///    },
    ///    "bookmark": {
    ///      "$ref": "#/components/schemas/ExistingBookmark"
    ///    },
    ///    "tags": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ExistingTag"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct AnnotatedBookmark {
        pub associations: Vec<AssociatedLink>,
        pub bookmark: ExistingBookmark,
        pub tags: Vec<ExistingTag>,
    }
    impl From<&AnnotatedBookmark> for AnnotatedBookmark {
        fn from(value: &AnnotatedBookmark) -> Self {
            value.clone()
        }
    }
    impl AnnotatedBookmark {
        pub fn builder() -> builder::AnnotatedBookmark {
            Default::default()
        }
    }
    /**A link associated with a bookmark.

    Links can have a "context" in which that association happens
    (free-form text, given by the user), and they point to a URL,
    which in turn can be another bookmark.*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A link associated with a bookmark.\n\nLinks can have a \"context\" in which that association happens\n(free-form text, given by the user), and they point to a URL,\nwhich in turn can be another bookmark.",
    ///  "type": "object",
    ///  "required": [
    ///    "link"
    ///  ],
    ///  "properties": {
    ///    "context": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "link": {
    ///      "type": "string",
    ///      "format": "uri"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct AssociatedLink {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub context: Option<String>,
        pub link: String,
    }
    impl From<&AssociatedLink> for AssociatedLink {
        fn from(value: &AssociatedLink) -> Self {
            value.clone()
        }
    }
    impl AssociatedLink {
        pub fn builder() -> builder::AssociatedLink {
            Default::default()
        }
    }
    ///The database ID of a bookmark.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The database ID of a bookmark.",
    ///  "type": "integer",
    ///  "format": "int64"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct BookmarkId(pub i64);
    impl std::ops::Deref for BookmarkId {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl From<BookmarkId> for i64 {
        fn from(value: BookmarkId) -> Self {
            value.0
        }
    }
    impl From<&BookmarkId> for BookmarkId {
        fn from(value: &BookmarkId) -> Self {
            value.clone()
        }
    }
    impl From<i64> for BookmarkId {
        fn from(value: i64) -> Self {
            Self(value)
        }
    }
    impl std::str::FromStr for BookmarkId {
        type Err = <i64 as std::str::FromStr>::Err;
        fn from_str(value: &str) -> Result<Self, Self::Err> {
            Ok(Self(value.parse()?))
        }
    }
    impl std::convert::TryFrom<&str> for BookmarkId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for BookmarkId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for BookmarkId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl ToString for BookmarkId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    /**The possible criteria that we can search for in a bookmark
    query. See [BookmarkSearchCriteria].*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The possible criteria that we can search for in a bookmark\nquery. See [BookmarkSearchCriteria].",
    ///  "oneOf": [
    ///    {
    ///      "description": "Only list bookmarks that have matching timestamps",
    ///      "type": "object",
    ///      "required": [
    ///        "date"
    ///      ],
    ///      "properties": {
    ///        "date": {
    ///          "$ref": "#/components/schemas/BookmarkSearchDateParams"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "description": "Only list bookmarks that are tagged with a tag of the given name.",
    ///      "type": "object",
    ///      "required": [
    ///        "tag"
    ///      ],
    ///      "properties": {
    ///        "tag": {
    ///          "$ref": "#/components/schemas/TagName"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "description": "Only list bookmarks that are tagged with a tag with the given ID.",
    ///      "type": "object",
    ///      "required": [
    ///        "tag_id"
    ///      ],
    ///      "properties": {
    ///        "tag_id": {
    ///          "$ref": "#/components/schemas/TagId"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "description": "Only list bookmarks belonging to the given user.",
    ///      "type": "object",
    ///      "required": [
    ///        "user_id"
    ///      ],
    ///      "properties": {
    ///        "user_id": {
    ///          "$ref": "#/components/schemas/UserId"
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub enum BookmarkSearch {
        ///Only list bookmarks that have matching timestamps
        #[serde(rename = "date")]
        Date(BookmarkSearchDateParams),
        ///Only list bookmarks that are tagged with a tag of the given name.
        #[serde(rename = "tag")]
        Tag(TagName),
        ///Only list bookmarks that are tagged with a tag with the given ID.
        #[serde(rename = "tag_id")]
        TagId(TagId),
        ///Only list bookmarks belonging to the given user.
        #[serde(rename = "user_id")]
        UserId(UserId),
    }
    impl From<&BookmarkSearch> for BookmarkSearch {
        fn from(value: &BookmarkSearch) -> Self {
            value.clone()
        }
    }
    impl From<BookmarkSearchDateParams> for BookmarkSearch {
        fn from(value: BookmarkSearchDateParams) -> Self {
            Self::Date(value)
        }
    }
    impl From<TagName> for BookmarkSearch {
        fn from(value: TagName) -> Self {
            Self::Tag(value)
        }
    }
    impl From<TagId> for BookmarkSearch {
        fn from(value: TagId) -> Self {
            Self::TagId(value)
        }
    }
    impl From<UserId> for BookmarkSearch {
        fn from(value: UserId) -> Self {
            Self::UserId(value)
        }
    }
    ///BookmarkSearchDateParams
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "date",
    ///    "field",
    ///    "orientation"
    ///  ],
    ///  "properties": {
    ///    "date": {
    ///      "$ref": "#/components/schemas/DateInput"
    ///    },
    ///    "field": {
    ///      "$ref": "#/components/schemas/BookmarkSearchDatetimeField"
    ///    },
    ///    "orientation": {
    ///      "$ref": "#/components/schemas/BookmarkSearchDatetimeOrientation"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct BookmarkSearchDateParams {
        pub date: DateInput,
        pub field: BookmarkSearchDatetimeField,
        pub orientation: BookmarkSearchDatetimeOrientation,
    }
    impl From<&BookmarkSearchDateParams> for BookmarkSearchDateParams {
        fn from(value: &BookmarkSearchDateParams) -> Self {
            value.clone()
        }
    }
    impl BookmarkSearchDateParams {
        pub fn builder() -> builder::BookmarkSearchDateParams {
            Default::default()
        }
    }
    ///BookmarkSearchDatetimeField
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "Created"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub enum BookmarkSearchDatetimeField {
        Created,
    }
    impl From<&BookmarkSearchDatetimeField> for BookmarkSearchDatetimeField {
        fn from(value: &BookmarkSearchDatetimeField) -> Self {
            value.clone()
        }
    }
    impl ToString for BookmarkSearchDatetimeField {
        fn to_string(&self) -> String {
            match *self {
                Self::Created => "Created".to_string(),
            }
        }
    }
    impl std::str::FromStr for BookmarkSearchDatetimeField {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
            match value {
                "Created" => Ok(Self::Created),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl std::convert::TryFrom<&str> for BookmarkSearchDatetimeField {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for BookmarkSearchDatetimeField {
        type Error = self::error::ConversionError;
        fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for BookmarkSearchDatetimeField {
        type Error = self::error::ConversionError;
        fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///BookmarkSearchDatetimeOrientation
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "After",
    ///    "Before"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub enum BookmarkSearchDatetimeOrientation {
        After,
        Before,
    }
    impl From<&BookmarkSearchDatetimeOrientation> for BookmarkSearchDatetimeOrientation {
        fn from(value: &BookmarkSearchDatetimeOrientation) -> Self {
            value.clone()
        }
    }
    impl ToString for BookmarkSearchDatetimeOrientation {
        fn to_string(&self) -> String {
            match *self {
                Self::After => "After".to_string(),
                Self::Before => "Before".to_string(),
            }
        }
    }
    impl std::str::FromStr for BookmarkSearchDatetimeOrientation {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
            match value {
                "After" => Ok(Self::After),
                "Before" => Ok(Self::Before),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl std::convert::TryFrom<&str> for BookmarkSearchDatetimeOrientation {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for BookmarkSearchDatetimeOrientation {
        type Error = self::error::ConversionError;
        fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for BookmarkSearchDatetimeOrientation {
        type Error = self::error::ConversionError;
        fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///DateInput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct DateInput(pub String);
    impl std::ops::Deref for DateInput {
        type Target = String;
        fn deref(&self) -> &String {
            &self.0
        }
    }
    impl From<DateInput> for String {
        fn from(value: DateInput) -> Self {
            value.0
        }
    }
    impl From<&DateInput> for DateInput {
        fn from(value: &DateInput) -> Self {
            value.clone()
        }
    }
    impl From<String> for DateInput {
        fn from(value: String) -> Self {
            Self(value)
        }
    }
    impl std::str::FromStr for DateInput {
        type Err = std::convert::Infallible;
        fn from_str(value: &str) -> Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ToString for DateInput {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    /**A bookmark saved by a user.

    See the section in [Transaction][Transaction#working-with-bookmarks]*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A bookmark saved by a user.\n\nSee the section in [Transaction][Transaction#working-with-bookmarks]",
    ///  "type": "object",
    ///  "required": [
    ///    "created_at",
    ///    "id",
    ///    "shared",
    ///    "title",
    ///    "unread",
    ///    "url",
    ///    "user_id"
    ///  ],
    ///  "properties": {
    ///    "accessed_at": {
    ///      "description": "Last time the bookmark was accessed via the web",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ],
    ///      "format": "date-time"
    ///    },
    ///    "created_at": {
    ///      "description": "Time at which the bookmark was created.\n\nThis time is assigned in code here, not in the database.",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "description": {
    ///      "description": "Description of the bookmark, possibly extracted from the website.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "id": {
    ///      "$ref": "#/components/schemas/BookmarkId"
    ///    },
    ///    "modified_at": {
    ///      "description": "Last time the bookmark was modified.\n\nThis field indicates modifications to the bookmark data itself\nonly, not changes to tags or related models.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ],
    ///      "format": "date-time"
    ///    },
    ///    "notes": {
    ///      "description": "Private notes that the user attached to the bookmark.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "shared": {
    ///      "description": "Whether other users can see the bookmark.",
    ///      "type": "boolean"
    ///    },
    ///    "title": {
    ///      "description": "Title that the user gave the bookmark.",
    ///      "type": "string"
    ///    },
    ///    "unread": {
    ///      "description": "Whether the bookmark is \"to read\"",
    ///      "type": "boolean"
    ///    },
    ///    "url": {
    ///      "description": "URL that the bookmark points to.",
    ///      "type": "string",
    ///      "format": "uri"
    ///    },
    ///    "user_id": {
    ///      "$ref": "#/components/schemas/UserId"
    ///    },
    ///    "website_description": {
    ///      "description": "Original description extracted from the website.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "website_title": {
    ///      "description": "Original title extracted from the website.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct ExistingBookmark {
        ///Last time the bookmark was accessed via the web
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub accessed_at: Option<chrono::DateTime<chrono::offset::Utc>>,
        /**Time at which the bookmark was created.

        This time is assigned in code here, not in the database.*/
        pub created_at: chrono::DateTime<chrono::offset::Utc>,
        ///Description of the bookmark, possibly extracted from the website.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub id: BookmarkId,
        /**Last time the bookmark was modified.

        This field indicates modifications to the bookmark data itself
        only, not changes to tags or related models.*/
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub modified_at: Option<chrono::DateTime<chrono::offset::Utc>>,
        ///Private notes that the user attached to the bookmark.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub notes: Option<String>,
        ///Whether other users can see the bookmark.
        pub shared: bool,
        ///Title that the user gave the bookmark.
        pub title: String,
        ///Whether the bookmark is "to read"
        pub unread: bool,
        ///URL that the bookmark points to.
        pub url: String,
        pub user_id: UserId,
        ///Original description extracted from the website.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub website_description: Option<String>,
        ///Original title extracted from the website.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub website_title: Option<String>,
    }
    impl From<&ExistingBookmark> for ExistingBookmark {
        fn from(value: &ExistingBookmark) -> Self {
            value.clone()
        }
    }
    impl ExistingBookmark {
        pub fn builder() -> builder::ExistingBookmark {
            Default::default()
        }
    }
    /**A named tag, possibly assigned to multiple bookmarks.

    See the section in [Transaction][Transaction#working-with-tags]*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A named tag, possibly assigned to multiple bookmarks.\n\nSee the section in [Transaction][Transaction#working-with-tags]",
    ///  "type": "object",
    ///  "required": [
    ///    "created_at",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "created_at": {
    ///      "description": "When the tag was first created.",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "name": {
    ///      "description": "Name of the tag.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct ExistingTag {
        ///When the tag was first created.
        pub created_at: chrono::DateTime<chrono::offset::Utc>,
        ///Name of the tag.
        pub name: String,
    }
    impl From<&ExistingTag> for ExistingTag {
        fn from(value: &ExistingTag) -> Self {
            value.clone()
        }
    }
    impl ExistingTag {
        pub fn builder() -> builder::ExistingTag {
            Default::default()
        }
    }
    /**The response returned by the `list_bookmarks` API endpoint.

    This response contains pagination information; if `next_cursor` is
    set, passing that value to the `cursor` pagination parameter will
    fetch the next page.*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The response returned by the `list_bookmarks` API endpoint.\n\nThis response contains pagination information; if `next_cursor` is\nset, passing that value to the `cursor` pagination parameter will\nfetch the next page.",
    ///  "type": "object",
    ///  "required": [
    ///    "bookmarks"
    ///  ],
    ///  "properties": {
    ///    "bookmarks": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AnnotatedBookmark"
    ///      }
    ///    },
    ///    "next_cursor": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/BookmarkId"
    ///        }
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct ListBookmarkResult {
        pub bookmarks: Vec<AnnotatedBookmark>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub next_cursor: Option<BookmarkId>,
    }
    impl From<&ListBookmarkResult> for ListBookmarkResult {
        fn from(value: &ListBookmarkResult) -> Self {
            value.clone()
        }
    }
    impl ListBookmarkResult {
        pub fn builder() -> builder::ListBookmarkResult {
            Default::default()
        }
    }
    /**The response returned by the `list_bookmarks` API endpoint.

    This response contains pagination information; if `next_cursor` is
    set, passing that value to the `cursor` pagination parameter will
    fetch the next page.*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The response returned by the `list_bookmarks` API endpoint.\n\nThis response contains pagination information; if `next_cursor` is\nset, passing that value to the `cursor` pagination parameter will\nfetch the next page.",
    ///  "type": "object",
    ///  "required": [
    ///    "bookmarks"
    ///  ],
    ///  "properties": {
    ///    "bookmarks": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AnnotatedBookmark"
    ///      }
    ///    },
    ///    "next_cursor": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/BookmarkId"
    ///        }
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct ListBookmarksMatchingResponse {
        pub bookmarks: Vec<AnnotatedBookmark>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub next_cursor: Option<BookmarkId>,
    }
    impl From<&ListBookmarksMatchingResponse> for ListBookmarksMatchingResponse {
        fn from(value: &ListBookmarksMatchingResponse) -> Self {
            value.clone()
        }
    }
    impl ListBookmarksMatchingResponse {
        pub fn builder() -> builder::ListBookmarksMatchingResponse {
            Default::default()
        }
    }
    ///A bookmark search query request
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A bookmark search query request",
    ///  "allOf": [
    ///    {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/Pagination"
    ///        }
    ///      ]
    ///    },
    ///    {
    ///      "type": "object",
    ///      "properties": {
    ///        "query": {
    ///          "description": "A search of criteria, restricting the set of bookmarks that qualify.\n\nAll criteria are merged using logical AND / set intersection.",
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/BookmarkSearch"
    ///          }
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct ListRequest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<BookmarkId>,
        ///How many items to return
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub per_page: Option<i64>,
        /**A search of criteria, restricting the set of bookmarks that qualify.

        All criteria are merged using logical AND / set intersection.*/
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub query: Vec<BookmarkSearch>,
    }
    impl From<&ListRequest> for ListRequest {
        fn from(value: &ListRequest) -> Self {
            value.clone()
        }
    }
    impl ListRequest {
        pub fn builder() -> builder::ListRequest {
            Default::default()
        }
    }
    /**Parameters that govern non-offset based pagination.

    Pagination in `lz` works by getting the next page based on what
    the previous page's last element was, aka "cursor-based
    pagination". To that end, use the previous call's `nextCursor`
    parameter into this call's `cursor` parameter.*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Parameters that govern non-offset based pagination.\n\nPagination in `lz` works by getting the next page based on what\nthe previous page's last element was, aka \"cursor-based\npagination\". To that end, use the previous call's `nextCursor`\nparameter into this call's `cursor` parameter.",
    ///  "type": "object",
    ///  "properties": {
    ///    "cursor": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/BookmarkId"
    ///        }
    ///      ]
    ///    },
    ///    "per_page": {
    ///      "description": "How many items to return",
    ///      "examples": [
    ///        50
    ///      ],
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "format": "int32",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct Pagination {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<BookmarkId>,
        ///How many items to return
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub per_page: Option<i64>,
    }
    impl From<&Pagination> for Pagination {
        fn from(value: &Pagination) -> Self {
            value.clone()
        }
    }
    impl Pagination {
        pub fn builder() -> builder::Pagination {
            Default::default()
        }
    }
    ///The database ID of a tag.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The database ID of a tag.",
    ///  "type": "integer",
    ///  "format": "int64"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct TagId(pub i64);
    impl std::ops::Deref for TagId {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl From<TagId> for i64 {
        fn from(value: TagId) -> Self {
            value.0
        }
    }
    impl From<&TagId> for TagId {
        fn from(value: &TagId) -> Self {
            value.clone()
        }
    }
    impl From<i64> for TagId {
        fn from(value: i64) -> Self {
            Self(value)
        }
    }
    impl std::str::FromStr for TagId {
        type Err = <i64 as std::str::FromStr>::Err;
        fn from_str(value: &str) -> Result<Self, Self::Err> {
            Ok(Self(value.parse()?))
        }
    }
    impl std::convert::TryFrom<&str> for TagId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for TagId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for TagId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl ToString for TagId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    ///The name representation of a tag.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The name representation of a tag.",
    ///  "type": "string"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    pub struct TagName(pub String);
    impl std::ops::Deref for TagName {
        type Target = String;
        fn deref(&self) -> &String {
            &self.0
        }
    }
    impl From<TagName> for String {
        fn from(value: TagName) -> Self {
            value.0
        }
    }
    impl From<&TagName> for TagName {
        fn from(value: &TagName) -> Self {
            value.clone()
        }
    }
    impl From<String> for TagName {
        fn from(value: String) -> Self {
            Self(value)
        }
    }
    impl std::str::FromStr for TagName {
        type Err = std::convert::Infallible;
        fn from_str(value: &str) -> Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ToString for TagName {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    /**A search query for retrieving bookmarks via the tags assigned to them.

    Each*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A search query for retrieving bookmarks via the tags assigned to them.\n\nEach",
    ///  "type": "object",
    ///  "properties": {
    ///    "tags": {
    ///      "description": "Tags that all returned items should have.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/TagName"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct TagQuery {
        ///Tags that all returned items should have.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tags: Vec<TagName>,
    }
    impl From<&TagQuery> for TagQuery {
        fn from(value: &TagQuery) -> Self {
            value.clone()
        }
    }
    impl TagQuery {
        pub fn builder() -> builder::TagQuery {
            Default::default()
        }
    }
    ///The database ID of a user.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The database ID of a user.",
    ///  "type": "integer",
    ///  "format": "int64"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct UserId(pub i64);
    impl std::ops::Deref for UserId {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl From<UserId> for i64 {
        fn from(value: UserId) -> Self {
            value.0
        }
    }
    impl From<&UserId> for UserId {
        fn from(value: &UserId) -> Self {
            value.clone()
        }
    }
    impl From<i64> for UserId {
        fn from(value: i64) -> Self {
            Self(value)
        }
    }
    impl std::str::FromStr for UserId {
        type Err = <i64 as std::str::FromStr>::Err;
        fn from_str(value: &str) -> Result<Self, Self::Err> {
            Ok(Self(value.parse()?))
        }
    }
    impl std::convert::TryFrom<&str> for UserId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for UserId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: &String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for UserId {
        type Error = <i64 as std::str::FromStr>::Err;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl ToString for UserId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct AnnotatedBookmark {
            associations: Result<Vec<super::AssociatedLink>, String>,
            bookmark: Result<super::ExistingBookmark, String>,
            tags: Result<Vec<super::ExistingTag>, String>,
        }
        impl Default for AnnotatedBookmark {
            fn default() -> Self {
                Self {
                    associations: Err("no value supplied for associations".to_string()),
                    bookmark: Err("no value supplied for bookmark".to_string()),
                    tags: Err("no value supplied for tags".to_string()),
                }
            }
        }
        impl AnnotatedBookmark {
            pub fn associations<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::AssociatedLink>>,
                T::Error: std::fmt::Display,
            {
                self.associations = value.try_into().map_err(|e| {
                    format!("error converting supplied value for associations: {}", e)
                });
                self
            }
            pub fn bookmark<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::ExistingBookmark>,
                T::Error: std::fmt::Display,
            {
                self.bookmark = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for bookmark: {}", e));
                self
            }
            pub fn tags<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::ExistingTag>>,
                T::Error: std::fmt::Display,
            {
                self.tags = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tags: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<AnnotatedBookmark> for super::AnnotatedBookmark {
            type Error = super::error::ConversionError;
            fn try_from(value: AnnotatedBookmark) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    associations: value.associations?,
                    bookmark: value.bookmark?,
                    tags: value.tags?,
                })
            }
        }
        impl From<super::AnnotatedBookmark> for AnnotatedBookmark {
            fn from(value: super::AnnotatedBookmark) -> Self {
                Self {
                    associations: Ok(value.associations),
                    bookmark: Ok(value.bookmark),
                    tags: Ok(value.tags),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct AssociatedLink {
            context: Result<Option<String>, String>,
            link: Result<String, String>,
        }
        impl Default for AssociatedLink {
            fn default() -> Self {
                Self {
                    context: Ok(Default::default()),
                    link: Err("no value supplied for link".to_string()),
                }
            }
        }
        impl AssociatedLink {
            pub fn context<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<String>>,
                T::Error: std::fmt::Display,
            {
                self.context = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for context: {}", e));
                self
            }
            pub fn link<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<String>,
                T::Error: std::fmt::Display,
            {
                self.link = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for link: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<AssociatedLink> for super::AssociatedLink {
            type Error = super::error::ConversionError;
            fn try_from(value: AssociatedLink) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    context: value.context?,
                    link: value.link?,
                })
            }
        }
        impl From<super::AssociatedLink> for AssociatedLink {
            fn from(value: super::AssociatedLink) -> Self {
                Self {
                    context: Ok(value.context),
                    link: Ok(value.link),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct BookmarkSearchDateParams {
            date: Result<super::DateInput, String>,
            field: Result<super::BookmarkSearchDatetimeField, String>,
            orientation: Result<super::BookmarkSearchDatetimeOrientation, String>,
        }
        impl Default for BookmarkSearchDateParams {
            fn default() -> Self {
                Self {
                    date: Err("no value supplied for date".to_string()),
                    field: Err("no value supplied for field".to_string()),
                    orientation: Err("no value supplied for orientation".to_string()),
                }
            }
        }
        impl BookmarkSearchDateParams {
            pub fn date<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::DateInput>,
                T::Error: std::fmt::Display,
            {
                self.date = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for date: {}", e));
                self
            }
            pub fn field<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::BookmarkSearchDatetimeField>,
                T::Error: std::fmt::Display,
            {
                self.field = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for field: {}", e));
                self
            }
            pub fn orientation<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::BookmarkSearchDatetimeOrientation>,
                T::Error: std::fmt::Display,
            {
                self.orientation = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for orientation: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<BookmarkSearchDateParams> for super::BookmarkSearchDateParams {
            type Error = super::error::ConversionError;
            fn try_from(
                value: BookmarkSearchDateParams,
            ) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    date: value.date?,
                    field: value.field?,
                    orientation: value.orientation?,
                })
            }
        }
        impl From<super::BookmarkSearchDateParams> for BookmarkSearchDateParams {
            fn from(value: super::BookmarkSearchDateParams) -> Self {
                Self {
                    date: Ok(value.date),
                    field: Ok(value.field),
                    orientation: Ok(value.orientation),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct ExistingBookmark {
            accessed_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>,
            created_at: Result<chrono::DateTime<chrono::offset::Utc>, String>,
            description: Result<Option<String>, String>,
            id: Result<super::BookmarkId, String>,
            modified_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>,
            notes: Result<Option<String>, String>,
            shared: Result<bool, String>,
            title: Result<String, String>,
            unread: Result<bool, String>,
            url: Result<String, String>,
            user_id: Result<super::UserId, String>,
            website_description: Result<Option<String>, String>,
            website_title: Result<Option<String>, String>,
        }
        impl Default for ExistingBookmark {
            fn default() -> Self {
                Self {
                    accessed_at: Ok(Default::default()),
                    created_at: Err("no value supplied for created_at".to_string()),
                    description: Ok(Default::default()),
                    id: Err("no value supplied for id".to_string()),
                    modified_at: Ok(Default::default()),
                    notes: Ok(Default::default()),
                    shared: Err("no value supplied for shared".to_string()),
                    title: Err("no value supplied for title".to_string()),
                    unread: Err("no value supplied for unread".to_string()),
                    url: Err("no value supplied for url".to_string()),
                    user_id: Err("no value supplied for user_id".to_string()),
                    website_description: Ok(Default::default()),
                    website_title: Ok(Default::default()),
                }
            }
        }
        impl ExistingBookmark {
            pub fn accessed_at<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<chrono::DateTime<chrono::offset::Utc>>>,
                T::Error: std::fmt::Display,
            {
                self.accessed_at = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for accessed_at: {}", e));
                self
            }
            pub fn created_at<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<chrono::DateTime<chrono::offset::Utc>>,
                T::Error: std::fmt::Display,
            {
                self.created_at = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for created_at: {}", e));
                self
            }
            pub fn description<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<String>>,
                T::Error: std::fmt::Display,
            {
                self.description = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for description: {}", e));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::BookmarkId>,
                T::Error: std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {}", e));
                self
            }
            pub fn modified_at<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<chrono::DateTime<chrono::offset::Utc>>>,
                T::Error: std::fmt::Display,
            {
                self.modified_at = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for modified_at: {}", e));
                self
            }
            pub fn notes<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<String>>,
                T::Error: std::fmt::Display,
            {
                self.notes = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for notes: {}", e));
                self
            }
            pub fn shared<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<bool>,
                T::Error: std::fmt::Display,
            {
                self.shared = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for shared: {}", e));
                self
            }
            pub fn title<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<String>,
                T::Error: std::fmt::Display,
            {
                self.title = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for title: {}", e));
                self
            }
            pub fn unread<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<bool>,
                T::Error: std::fmt::Display,
            {
                self.unread = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for unread: {}", e));
                self
            }
            pub fn url<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<String>,
                T::Error: std::fmt::Display,
            {
                self.url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for url: {}", e));
                self
            }
            pub fn user_id<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<super::UserId>,
                T::Error: std::fmt::Display,
            {
                self.user_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for user_id: {}", e));
                self
            }
            pub fn website_description<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<String>>,
                T::Error: std::fmt::Display,
            {
                self.website_description = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for website_description: {}",
                        e
                    )
                });
                self
            }
            pub fn website_title<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<String>>,
                T::Error: std::fmt::Display,
            {
                self.website_title = value.try_into().map_err(|e| {
                    format!("error converting supplied value for website_title: {}", e)
                });
                self
            }
        }
        impl std::convert::TryFrom<ExistingBookmark> for super::ExistingBookmark {
            type Error = super::error::ConversionError;
            fn try_from(value: ExistingBookmark) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    accessed_at: value.accessed_at?,
                    created_at: value.created_at?,
                    description: value.description?,
                    id: value.id?,
                    modified_at: value.modified_at?,
                    notes: value.notes?,
                    shared: value.shared?,
                    title: value.title?,
                    unread: value.unread?,
                    url: value.url?,
                    user_id: value.user_id?,
                    website_description: value.website_description?,
                    website_title: value.website_title?,
                })
            }
        }
        impl From<super::ExistingBookmark> for ExistingBookmark {
            fn from(value: super::ExistingBookmark) -> Self {
                Self {
                    accessed_at: Ok(value.accessed_at),
                    created_at: Ok(value.created_at),
                    description: Ok(value.description),
                    id: Ok(value.id),
                    modified_at: Ok(value.modified_at),
                    notes: Ok(value.notes),
                    shared: Ok(value.shared),
                    title: Ok(value.title),
                    unread: Ok(value.unread),
                    url: Ok(value.url),
                    user_id: Ok(value.user_id),
                    website_description: Ok(value.website_description),
                    website_title: Ok(value.website_title),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct ExistingTag {
            created_at: Result<chrono::DateTime<chrono::offset::Utc>, String>,
            name: Result<String, String>,
        }
        impl Default for ExistingTag {
            fn default() -> Self {
                Self {
                    created_at: Err("no value supplied for created_at".to_string()),
                    name: Err("no value supplied for name".to_string()),
                }
            }
        }
        impl ExistingTag {
            pub fn created_at<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<chrono::DateTime<chrono::offset::Utc>>,
                T::Error: std::fmt::Display,
            {
                self.created_at = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for created_at: {}", e));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<String>,
                T::Error: std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<ExistingTag> for super::ExistingTag {
            type Error = super::error::ConversionError;
            fn try_from(value: ExistingTag) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    created_at: value.created_at?,
                    name: value.name?,
                })
            }
        }
        impl From<super::ExistingTag> for ExistingTag {
            fn from(value: super::ExistingTag) -> Self {
                Self {
                    created_at: Ok(value.created_at),
                    name: Ok(value.name),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct ListBookmarkResult {
            bookmarks: Result<Vec<super::AnnotatedBookmark>, String>,
            next_cursor: Result<Option<super::BookmarkId>, String>,
        }
        impl Default for ListBookmarkResult {
            fn default() -> Self {
                Self {
                    bookmarks: Err("no value supplied for bookmarks".to_string()),
                    next_cursor: Ok(Default::default()),
                }
            }
        }
        impl ListBookmarkResult {
            pub fn bookmarks<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::AnnotatedBookmark>>,
                T::Error: std::fmt::Display,
            {
                self.bookmarks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for bookmarks: {}", e));
                self
            }
            pub fn next_cursor<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<super::BookmarkId>>,
                T::Error: std::fmt::Display,
            {
                self.next_cursor = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for next_cursor: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<ListBookmarkResult> for super::ListBookmarkResult {
            type Error = super::error::ConversionError;
            fn try_from(value: ListBookmarkResult) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    bookmarks: value.bookmarks?,
                    next_cursor: value.next_cursor?,
                })
            }
        }
        impl From<super::ListBookmarkResult> for ListBookmarkResult {
            fn from(value: super::ListBookmarkResult) -> Self {
                Self {
                    bookmarks: Ok(value.bookmarks),
                    next_cursor: Ok(value.next_cursor),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct ListBookmarksMatchingResponse {
            bookmarks: Result<Vec<super::AnnotatedBookmark>, String>,
            next_cursor: Result<Option<super::BookmarkId>, String>,
        }
        impl Default for ListBookmarksMatchingResponse {
            fn default() -> Self {
                Self {
                    bookmarks: Err("no value supplied for bookmarks".to_string()),
                    next_cursor: Ok(Default::default()),
                }
            }
        }
        impl ListBookmarksMatchingResponse {
            pub fn bookmarks<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::AnnotatedBookmark>>,
                T::Error: std::fmt::Display,
            {
                self.bookmarks = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for bookmarks: {}", e));
                self
            }
            pub fn next_cursor<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<super::BookmarkId>>,
                T::Error: std::fmt::Display,
            {
                self.next_cursor = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for next_cursor: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<ListBookmarksMatchingResponse> for super::ListBookmarksMatchingResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ListBookmarksMatchingResponse,
            ) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    bookmarks: value.bookmarks?,
                    next_cursor: value.next_cursor?,
                })
            }
        }
        impl From<super::ListBookmarksMatchingResponse> for ListBookmarksMatchingResponse {
            fn from(value: super::ListBookmarksMatchingResponse) -> Self {
                Self {
                    bookmarks: Ok(value.bookmarks),
                    next_cursor: Ok(value.next_cursor),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct ListRequest {
            cursor: Result<Option<super::BookmarkId>, String>,
            per_page: Result<Option<i64>, String>,
            query: Result<Vec<super::BookmarkSearch>, String>,
        }
        impl Default for ListRequest {
            fn default() -> Self {
                Self {
                    cursor: Ok(Default::default()),
                    per_page: Ok(Default::default()),
                    query: Ok(Default::default()),
                }
            }
        }
        impl ListRequest {
            pub fn cursor<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<super::BookmarkId>>,
                T::Error: std::fmt::Display,
            {
                self.cursor = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cursor: {}", e));
                self
            }
            pub fn per_page<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<i64>>,
                T::Error: std::fmt::Display,
            {
                self.per_page = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for per_page: {}", e));
                self
            }
            pub fn query<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::BookmarkSearch>>,
                T::Error: std::fmt::Display,
            {
                self.query = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for query: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<ListRequest> for super::ListRequest {
            type Error = super::error::ConversionError;
            fn try_from(value: ListRequest) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    cursor: value.cursor?,
                    per_page: value.per_page?,
                    query: value.query?,
                })
            }
        }
        impl From<super::ListRequest> for ListRequest {
            fn from(value: super::ListRequest) -> Self {
                Self {
                    cursor: Ok(value.cursor),
                    per_page: Ok(value.per_page),
                    query: Ok(value.query),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct Pagination {
            cursor: Result<Option<super::BookmarkId>, String>,
            per_page: Result<Option<i64>, String>,
        }
        impl Default for Pagination {
            fn default() -> Self {
                Self {
                    cursor: Ok(Default::default()),
                    per_page: Ok(Default::default()),
                }
            }
        }
        impl Pagination {
            pub fn cursor<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<super::BookmarkId>>,
                T::Error: std::fmt::Display,
            {
                self.cursor = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cursor: {}", e));
                self
            }
            pub fn per_page<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Option<i64>>,
                T::Error: std::fmt::Display,
            {
                self.per_page = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for per_page: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<Pagination> for super::Pagination {
            type Error = super::error::ConversionError;
            fn try_from(value: Pagination) -> Result<Self, super::error::ConversionError> {
                Ok(Self {
                    cursor: value.cursor?,
                    per_page: value.per_page?,
                })
            }
        }
        impl From<super::Pagination> for Pagination {
            fn from(value: super::Pagination) -> Self {
                Self {
                    cursor: Ok(value.cursor),
                    per_page: Ok(value.per_page),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct TagQuery {
            tags: Result<Vec<super::TagName>, String>,
        }
        impl Default for TagQuery {
            fn default() -> Self {
                Self {
                    tags: Ok(Default::default()),
                }
            }
        }
        impl TagQuery {
            pub fn tags<T>(mut self, value: T) -> Self
            where
                T: std::convert::TryInto<Vec<super::TagName>>,
                T::Error: std::fmt::Display,
            {
                self.tags = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tags: {}", e));
                self
            }
        }
        impl std::convert::TryFrom<TagQuery> for super::TagQuery {
            type Error = super::error::ConversionError;
            fn try_from(value: TagQuery) -> Result<Self, super::error::ConversionError> {
                Ok(Self { tags: value.tags? })
            }
        }
        impl From<super::TagQuery> for TagQuery {
            fn from(value: super::TagQuery) -> Self {
                Self {
                    tags: Ok(value.tags),
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
/**Client for lz-web



Version: 0.1.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }
    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "0.1.0"
    }
}
impl Client {
    /**List the user's bookmarks matching a query, newest to oldest

    List the user's bookmarks matching a query, newest to oldest.

    Sends a `POST` request to `/bookmarks`

    Arguments:
    - `body`:
    ```ignore
    let response = client.list_bookmarks_matching()
        .body(body)
        .send()
        .await;
    ```*/
    pub fn list_bookmarks_matching(&self) -> builder::ListBookmarksMatching {
        builder::ListBookmarksMatching::new(self)
    }
}
/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        encode_path, ByteStream, Error, HeaderMap, HeaderValue, RequestBuilderExt, ResponseValue,
    };
    /**Builder for [`Client::list_bookmarks_matching`]

    [`Client::list_bookmarks_matching`]: super::Client::list_bookmarks_matching*/
    #[derive(Debug, Clone)]
    pub struct ListBookmarksMatching<'a> {
        client: &'a super::Client,
        body: Result<types::builder::ListRequest, String>,
    }
    impl<'a> ListBookmarksMatching<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(types::builder::ListRequest::default()),
            }
        }
        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ListRequest>,
            <V as std::convert::TryInto<types::ListRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `ListRequest` for body failed: {}", s));
            self
        }
        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::ListRequest) -> types::builder::ListRequest,
        {
            self.body = self.body.map(f);
            self
        }
        ///Sends a `POST` request to `/bookmarks`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::ListBookmarksMatchingResponse>, Error<()>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::ListRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/bookmarks", client.baseurl,);
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .build()?;
            let result = client.client.execute(request).await;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }
}
/// Items consumers will typically use such as the Client.
pub mod prelude {
    pub use self::super::Client;
}
