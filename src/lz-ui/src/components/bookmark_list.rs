use dioxus::prelude::*;
use lz_openapi::types::{BookmarkId, ListBookmarksMatchingResponse, ListRequest};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use crate::components::Bookmark;
use crate::{use_api_client, use_search_criteria, BookmarkQuery};

#[component]
pub(crate) fn BookmarkList() -> Element {
    let next = use_signal(|| LoadNext::Load(None));
    rsx! {
        BookmarkCons { search: use_search_criteria(), next }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum LoadNext {
    Load(Option<BookmarkId>),
    Available(BookmarkId),
    Finished,
}

#[component]
fn BookmarkCons(search: Signal<BookmarkQuery>, next: Signal<LoadNext>) -> Element {
    let client = use_api_client();
    let current = *next.read();
    match current {
        LoadNext::Load(next_cursor) => {
            let batch = use_resource(move || {
                let query = search.read().to_criteria();
                info!(?query, "searching...");
                async move {
                    let query = ListRequest {
                        query,
                        cursor: next_cursor,
                        per_page: Default::default(),
                    };
                    let client = client.read().clone();
                    client.list_bookmarks_matching().body(query).send().await
                }
            });
            rsx! {
                match &*batch.value().read_unchecked() {
                    Some(Ok(ref batch)) => {
                        let ListBookmarksMatchingResponse {
                            next_cursor,
                            bookmarks,
                        } = &**batch;
                        let next = use_signal(move || if let Some(next_cursor) = next_cursor { LoadNext::Available(*next_cursor) } else { LoadNext::Finished });
                        let debug_search = format!("{:?} {:?}", search.read(), next.read());
                        rsx! {
                            section {
                                key: "{debug_search}",
                                h2 {
                                    "{debug_search}"
                                }
                                for abm in bookmarks {
                                    Bookmark { abm: abm.clone() }
                                }
                            }
                            BookmarkCons { search, next }
                        }
                    }
                    Some(Err(error)) => {
                        error!(%error, error_debug=?error, ?next_cursor, ?search,
                            "Could not load next batch");
                        rsx!{
                            p { "Can not load"}
                        }
                    }
                    None => {
                        rsx! { p { "Loading..."} }
                    }
                }
            }
        }
        LoadNext::Available(next_cursor) => rsx! {
            button {
                onclick: move |_ev| {
                    info!(? next_cursor, "loading more...");
                    *next.write() = LoadNext::Load(Some(next_cursor));
                },
                "Load More..."
            }
        },
        LoadNext::Finished => None,
    }
}
