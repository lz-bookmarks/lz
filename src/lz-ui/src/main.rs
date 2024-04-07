#![allow(non_snake_case)]

use std::ops::Deref;

use dioxus::prelude::*;
use lz_openapi::types::{AnnotatedBookmark, BookmarkId, ListBookmarksResponse};
use tracing::Level;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

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
        .map(|w| w.location().href().unwrap() + "api/v1")
        .unwrap();
    use_context_provider(move || Signal::new(ApiClient(lz_openapi::Client::new(&base_url))));
    rsx! {
        Router::<Route> {}
    }
}

/// A context-available signal that contains an API client for the lz server.
#[derive(Clone)]
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

#[component]
fn Home() -> Element {
    let load_state = use_signal(|| BookmarkListState::default());
    let endless_bookmarks = use_signal(|| vec![]);
    let load_next_batch = use_bookmark_list(load_state, endless_bookmarks);

    rsx! {
        div {
            h1 { "My Bookmarks" }
            section {
                for batch in &*endless_bookmarks.read() {
                    section {
                        for abm in batch {
                            Bookmark { abm: abm.clone() }
                        }
                    }
                }
            }
            match *load_state.read() {
                BookmarkListState::Initial | BookmarkListState::Loading => rsx! { p { "Loading..." }},
                BookmarkListState::MoreAvailable(next_cursor) => rsx! {
                    button {
                        onclick: move |ev| {
                            info!(?ev, ?next_cursor);
                            load_next_batch.send(());
                        },
                        "Load more..."
                    }
                },
                _ => {info!("other!"); None}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum BookmarkListState {
    #[default]
    Initial,
    Loading,
    Finished,
    MoreAvailable(BookmarkId),
}

fn use_bookmark_list(
    mut state: Signal<BookmarkListState>,
    mut batches: Signal<Vec<Vec<AnnotatedBookmark>>>,
) -> Coroutine<()> {
    use futures::StreamExt as _;
    let load_task = use_coroutine(|mut rx: UnboundedReceiver<Option<BookmarkId>>| async move {
        while *state.read() != BookmarkListState::Finished {
            let Some(next_cursor) = rx.next().await else {
                break;
            };
            let original_state = state.read().clone();
            let client = use_api_client().read().clone();
            *state.write() = BookmarkListState::Loading;
            let mut list_call = client.list_bookmarks();
            if let Some(next_cursor) = &next_cursor {
                list_call = list_call.cursor(next_cursor.clone());
            }
            match list_call.send().await {
                Ok(response) => {
                    let ListBookmarksResponse {
                        next_cursor,
                        bookmarks,
                    } = &*response;
                    *state.write() = match next_cursor {
                        Some(next_cursor) => BookmarkListState::MoreAvailable(next_cursor.clone()),
                        None => BookmarkListState::Finished,
                    };
                    batches.write().push(bookmarks.clone());
                }
                Err(error) => {
                    error!(%error, error_debug=?error, ?next_cursor, "could not load bookmarks from offset");
                    *state.write() = original_state;
                    continue;
                }
            }
        }
    });
    let next_task = use_coroutine(|mut rx: UnboundedReceiver<()>| async move {
        load_task.send(None); // kick off loading the batches
        while let Some(_) = rx.next().await {
            match *state.read() {
                BookmarkListState::MoreAvailable(cursor) => load_task.send(Some(cursor.clone())),
                _ => break,
            }
        }
    });
    next_task
}

#[component]
fn Bookmark(abm: AnnotatedBookmark) -> Element {
    rsx! {
        article {
            span { "{abm.bookmark.created_at} " }
            a {
                href: "{abm.bookmark.url}",
                "{abm.bookmark.title}"
            }
            if let Some(description) = abm.bookmark.description {
                blockquote { "{description}" }
            }
        }
    }
}
