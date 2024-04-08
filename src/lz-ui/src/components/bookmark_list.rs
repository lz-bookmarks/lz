use dioxus::prelude::*;
use lz_openapi::types::{
    AnnotatedBookmark, BookmarkId, ListBookmarksMatchingResponse, ListRequest,
};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use crate::components::Bookmark;
use crate::use_api_client;

#[component]
pub(crate) fn BookmarkList() -> Element {
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
                BookmarkListState::Initial | BookmarkListState::Loading => rsx! {
                    p { "Loading..." }
                },
                BookmarkListState::MoreAvailable(next_cursor) => rsx! {
                    button {
                        onclick: move |ev| {
                            info!(?ev, ?next_cursor);
                            load_next_batch.send(());
                        },
                        "Load more..."
                    }
                },
                BookmarkListState::Finished => None,
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
    let client = use_api_client().read().clone();
    let load_task = use_coroutine(|mut rx: UnboundedReceiver<Option<BookmarkId>>| async move {
        while *state.read() != BookmarkListState::Finished {
            let Some(next_cursor) = rx.next().await else {
                break;
            };
            let original_state = state.read().clone();
            *state.write() = BookmarkListState::Loading;
            let query = ListRequest {
                query: vec![],
                cursor: next_cursor,
                per_page: Default::default(),
            };
            let list_call = client.list_bookmarks_matching().body(query);
            match list_call.send().await {
                Ok(response) => {
                    let ListBookmarksMatchingResponse {
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
