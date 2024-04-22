use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::BounceStates;
use lz_openapi::types::builder::ListRequest;
use lz_openapi::types::{BookmarkId, BookmarkSearch, ListBookmarksMatchingResponse};
use yew::prelude::*;

use crate::{components::*, GoddamnIt};

#[derive(Properties, Default, PartialEq, Clone, Eq, Hash, Debug)]
pub struct BookmarksProps {
    pub cursor: Option<BookmarkId>,
    pub query: Vec<BookmarkSearch>,
}

impl BookmarksProps {
    fn as_body(&self) -> ListRequest {
        ListRequest::default()
            .cursor(self.cursor)
            .query(self.query.clone())
    }
}

#[derive(PartialEq, Debug, Clone)]
struct BookmarkBatch(ListBookmarksMatchingResponse);

impl Deref for BookmarkBatch {
    type Target = ListBookmarksMatchingResponse;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait(?Send)]
impl Query for BookmarkBatch {
    type Input = BookmarksProps;
    type Error = GoddamnIt;

    async fn query(_states: &BounceStates, input: Rc<BookmarksProps>) -> QueryResult<Self> {
        let loc = web_sys::window().unwrap().location();
        let base_url = format!(
            "{}//{}/api/v1",
            loc.protocol().unwrap(),
            loc.host().unwrap()
        );

        let client = lz_openapi::Client::new(&base_url);
        let response = client
            .list_bookmarks_matching()
            .body(input.as_body())
            .send()
            .await
            .map_err(GoddamnIt::new)?;
        Ok(BookmarkBatch(response.into_inner()).into())
    }
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &BookmarksProps) -> Html {
    let load_next = use_state(|| false);
    let bookmarks = use_query_value::<BookmarkBatch>(Rc::new(props.clone()));
    match bookmarks.result() {
        None => html! { <span class="loading loading-dots" /> },
        Some(Ok(b)) => {
            let bookmark_items =
                b.0.bookmarks
                    .iter()
                    .map(|b| html! { <Bookmark bookmark={b.clone()} /> })
                    .collect::<Html>();
            html! {
                <section>
                    <>
                        { bookmark_items }
                        if let Some(next) = b.next_cursor {
                            if !*load_next {
                                <button
                                    class="btn btn-block"
                                    onclick={move |_ev| {
                                      load_next.set(true);
                                    }}
                                >
                                    { "Load more..." }
                                </button>
                            } else {
                                <Bookmarks cursor={next} query={props.query.clone()} />
                            }
                        }
                    </>
                </section>
            }
        }
        Some(Err(e)) => html! { <h1>{ e.to_string() }</h1> },
    }
}
