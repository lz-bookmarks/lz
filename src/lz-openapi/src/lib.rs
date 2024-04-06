mod progenitor_client;

#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
pub mod types {
    use serde::{Deserialize, Serialize};
    #[allow(unused_imports)]
    use std::convert::TryFrom;
    ///A bookmark, including tags and associations on it.
    #[derive(Clone, Debug, Deserialize, Serialize)]
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

    ///A link associated with a bookmark.
    ///
    ///Links can have a "context" in which that association happens
    ///(free-form text, given by the user), and they point to a URL,
    ///which in turn can be another bookmark.
    #[derive(Clone, Debug, Deserialize, Serialize)]
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

    ///The database ID of a bookmark.
    #[derive(Clone, Debug, Deserialize, Serialize)]
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

    ///A bookmark saved by a user.
    ///
    ///See the section in [Transaction][Transaction#working-with-bookmarks]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ExistingBookmark {
        ///Last time the bookmark was accessed via the web
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub accessed_at: Option<chrono::DateTime<chrono::offset::Utc>>,
        ///Time at which the bookmark was created.
        ///
        ///This time is assigned in code here, not in the database.
        pub created_at: chrono::DateTime<chrono::offset::Utc>,
        ///Description of the bookmark, possibly extracted from the website.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub id: BookmarkId,
        ///Last time the bookmark was modified.
        ///
        ///This field indicates modifications to the bookmark data itself
        ///only, not changes to tags or related models.
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

    ///A named tag, possibly assigned to multiple bookmarks.
    ///
    ///See the section in [Transaction][Transaction#working-with-tags]
    #[derive(Clone, Debug, Deserialize, Serialize)]
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

    ///The response returned by the `list_bookmarks` API endpoint.
    ///
    ///This response contains pagination information; if `next_cursor` is
    ///set, passing that value to the `cursor` pagination parameter will
    ///fetch the next page.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ListBookmarkResult {
        pub bookmarks: Vec<AnnotatedBookmark>,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub next_cursor: Option<BookmarkId>,
    }

    impl From<&ListBookmarkResult> for ListBookmarkResult {
        fn from(value: &ListBookmarkResult) -> Self {
            value.clone()
        }
    }

    ///Parameters that govern non-offset based pagination.
    ///
    ///Pagination in `lz` works by getting the next page based on what
    ///the previous page's last element was, aka "cursor-based
    ///pagination". To that end, use the previous call's `nextCursor`
    ///parameter into this call's `cursor` parameter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ListBookmarksPagination {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<BookmarkId>,
        ///How many items to return
        #[serde(rename = "perPage", default, skip_serializing_if = "Option::is_none")]
        pub per_page: Option<i64>,
    }

    impl From<&ListBookmarksPagination> for ListBookmarksPagination {
        fn from(value: &ListBookmarksPagination) -> Self {
            value.clone()
        }
    }

    ///The response returned by the `list_bookmarks` API endpoint.
    ///
    ///This response contains pagination information; if `next_cursor` is
    ///set, passing that value to the `cursor` pagination parameter will
    ///fetch the next page.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ListBookmarksResponse {
        pub bookmarks: Vec<AnnotatedBookmark>,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub next_cursor: Option<BookmarkId>,
    }

    impl From<&ListBookmarksResponse> for ListBookmarksResponse {
        fn from(value: &ListBookmarksResponse) -> Self {
            value.clone()
        }
    }

    ///Parameters that govern non-offset based pagination.
    ///
    ///Pagination in `lz` works by getting the next page based on what
    ///the previous page's last element was, aka "cursor-based
    ///pagination". To that end, use the previous call's `nextCursor`
    ///parameter into this call's `cursor` parameter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ListBookmarksWithTagPagination {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<BookmarkId>,
        ///How many items to return
        #[serde(rename = "perPage", default, skip_serializing_if = "Option::is_none")]
        pub per_page: Option<i64>,
    }

    impl From<&ListBookmarksWithTagPagination> for ListBookmarksWithTagPagination {
        fn from(value: &ListBookmarksWithTagPagination) -> Self {
            value.clone()
        }
    }

    ///The response returned by the `list_bookmarks` API endpoint.
    ///
    ///This response contains pagination information; if `next_cursor` is
    ///set, passing that value to the `cursor` pagination parameter will
    ///fetch the next page.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ListBookmarksWithTagResponse {
        pub bookmarks: Vec<AnnotatedBookmark>,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub next_cursor: Option<BookmarkId>,
    }

    impl From<&ListBookmarksWithTagResponse> for ListBookmarksWithTagResponse {
        fn from(value: &ListBookmarksWithTagResponse) -> Self {
            value.clone()
        }
    }

    ///Parameters that govern non-offset based pagination.
    ///
    ///Pagination in `lz` works by getting the next page based on what
    ///the previous page's last element was, aka "cursor-based
    ///pagination". To that end, use the previous call's `nextCursor`
    ///parameter into this call's `cursor` parameter.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Pagination {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<BookmarkId>,
        ///How many items to return
        #[serde(rename = "perPage", default, skip_serializing_if = "Option::is_none")]
        pub per_page: Option<i64>,
    }

    impl From<&Pagination> for Pagination {
        fn from(value: &Pagination) -> Self {
            value.clone()
        }
    }

    ///The name representation of a tag.
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

    ///A search query for retrieving bookmarks via the tags assigned to them.
    ///
    ///These tag queries are made in a URL path, separated by space
    ///(`%20`) characters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct TagQuery {
        ///Tags that all returned items should have.
        pub tags: Vec<TagName>,
    }

    impl From<&TagQuery> for TagQuery {
        fn from(value: &TagQuery) -> Self {
            value.clone()
        }
    }

    ///The database ID of a user.
    #[derive(Clone, Debug, Deserialize, Serialize)]
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
}

#[derive(Clone, Debug)]
///Client for lz-web
///
///
///
///Version: 0.1.0
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
    ///List the user's bookmarks, newest to oldest
    ///
    ///List the user's bookmarks, newest to oldest.
    ///
    ///Sends a `GET` request to `/bookmarks`
    pub async fn list_bookmarks<'a>(
        &'a self,
        pagination: Option<&'a types::ListBookmarksPagination>,
    ) -> Result<ResponseValue<types::ListBookmarksResponse>, Error<()>> {
        let url = format!("{}/bookmarks", self.baseurl,);
        let mut query = Vec::with_capacity(1usize);
        if let Some(v) = &pagination {
            query.push(("pagination", v.to_string()));
        }

        let request = self
            .client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&query)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///List bookmarks matching a tag, newest to oldest
    ///
    ///List bookmarks matching a tag, newest to oldest.
    ///
    ///Sends a `GET` request to `/bookmarks/tagged/{query}`
    pub async fn list_bookmarks_with_tag<'a>(
        &'a self,
        query: &'a str,
        pagination: Option<&'a types::ListBookmarksWithTagPagination>,
    ) -> Result<ResponseValue<types::ListBookmarksWithTagResponse>, Error<()>> {
        let url = format!(
            "{}/bookmarks/tagged/{}",
            self.baseurl,
            encode_path(&query.to_string()),
        );
        let mut query = Vec::with_capacity(1usize);
        if let Some(v) = &pagination {
            query.push(("pagination", v.to_string()));
        }

        let request = self
            .client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&query)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}

pub mod prelude {
    pub use super::Client;
}
