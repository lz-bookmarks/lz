#![allow(non_snake_case)]

use std::collections::HashSet;
use std::fmt;
use std::ops::Deref;

use dioxus::prelude::*;
use lz_openapi::types::{BookmarkSearch, TagName};
use tracing::Level;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use components::BookmarkList;

#[derive(Clone, Routable, Debug, PartialEq)]
pub(crate) enum Route {
    #[route("/:..search")]
    Bookmarks { search: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BookmarkQuery(HashSet<BookmarkSearch>);

impl FromRouteSegments for BookmarkQuery {
    fn from_route_segments(segments: &[&str]) -> Result<Self, Self::Err> {
        let val: Result<_, Self::Err> = segments
            .into_iter()
            .filter(|&&seg| seg != "")
            .map(|&segment| match segment.split_once(":") {
                None => Ok(BookmarkSearch::Tag(TagName(segment.to_string()))),
                Some(("tag", t)) => Ok(BookmarkSearch::Tag(TagName(t.to_string()))),
                // TODO: more bookmark query types!
                Some((name, value)) => Err(format!("Unable to match {:?}:{:?}", name, value)),
            })
            .collect();
        Ok(BookmarkQuery(val?))
    }

    type Err = String;
}

// TODO: This is unused because we can't use FromRouteSegments properly.
impl ToRouteSegments for BookmarkQuery {
    fn display_route_segments(self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.0 {
            write!(f, "/")?;
            let segment = BookmarkQuery::format_route_segment(&segment);
            match urlencoding::decode(&segment) {
                Ok(segment) => write!(f, "{}", segment)?,
                Err(err) => {
                    tracing::error!("Failed to decode url encoding: {}", err);
                    write!(f, "{}", segment)?
                }
            }
        }
        Ok(())
    }
}

impl BookmarkQuery {
    pub fn with_tag(&self, tag: &str) -> Self {
        let mut set = self.0.clone();
        set.insert(BookmarkSearch::Tag(TagName(tag.to_string())));
        BookmarkQuery(set)
    }

    pub fn to_criteria(&self) -> Vec<BookmarkSearch> {
        self.0.iter().cloned().collect()
    }

    fn format_route_segment(segment: &BookmarkSearch) -> String {
        match segment {
            BookmarkSearch::Date(_) => todo!(),
            BookmarkSearch::Tag(name) => format!("tag:{}", name.0),
            BookmarkSearch::TagId(id) => format!("tag_id:{}", id.0),
            BookmarkSearch::UserId(user_id) => format!("user_id:{}", user_id.0),
        }
    }

    pub fn to_route(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|s| Self::format_route_segment(s))
            .collect()
    }
}

#[component]
fn Bookmarks(search: Vec<String>) -> Element {
    let segments: Vec<&str> = search.iter().map(|s| s.as_str()).collect();
    if let Ok(search) = BookmarkQuery::from_route_segments(&segments.as_slice()) {
        rsx! {
            BookmarkList { search }
        }
    } else {
        error!("Could not generate route segments");
        None
    }
}

mod components;

fn main() {
    // Init debug
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(Level::DEBUG)
            .build(),
    );
    console_error_panic_hook::set_once();

    launch(App);
}

fn App() -> Element {
    let base_url = web_sys::window()
        .map(|w| {
            format!(
                "{}//{}/api/v1",
                w.location().protocol().unwrap(),
                w.location().host().unwrap()
            )
        })
        .unwrap();
    use_context_provider(move || Signal::new(ApiClient(lz_openapi::Client::new(&base_url))));
    rsx! {
        Router::<Route> {}
    }
}

/// A context-available signal that contains an API client for the lz server.
#[derive(Debug, Clone)]
pub(crate) struct ApiClient(lz_openapi::Client);

/// Use an API client.
/// ## Example usage
/// ```rust
/// let client = use_api_client().read().clone();
/// client.list_bookmarks().send().await
/// ```
pub(crate) fn use_api_client() -> Signal<ApiClient> {
    use_context()
}

impl Deref for ApiClient {
    type Target = lz_openapi::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Use the current of bookmark search criteria.
pub(crate) fn use_search_criteria() -> Signal<BookmarkQuery> {
    use_context()
}
