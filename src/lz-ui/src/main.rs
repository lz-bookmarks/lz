use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::{BounceRoot, BounceStates};
use lz_openapi::types::builder::ListRequest;
use lz_openapi::types::{AnnotatedBookmark, ListBookmarksMatchingResponse, Pagination};
use yew::prelude::*;

#[derive(PartialEq, Debug, Clone)]
struct BookmarkBatch(ListBookmarksMatchingResponse);

impl Deref for BookmarkBatch {
    type Target = ListBookmarksMatchingResponse;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", .0)]
struct GoddamnIt(String);

impl GoddamnIt {
    fn new<E>(error: E) -> Self
    where
        E: ToString,
    {
        GoddamnIt(error.to_string())
    }
}

#[async_trait(?Send)]
impl Query for BookmarkBatch {
    type Input = Pagination;
    type Error = GoddamnIt;

    async fn query(_states: &BounceStates, input: Rc<Self::Input>) -> QueryResult<Self> {
        let base_url = web_sys::window()
            .map(|w| w.location().href().unwrap() + "api/v1")
            .unwrap();

        let client = lz_openapi::Client::new(&base_url);
        let response = client
            .list_bookmarks_matching()
            .body(
                ListRequest::default()
                    .per_page(input.per_page)
                    .cursor(input.cursor),
            )
            .send()
            .await
            .map_err(GoddamnIt::new)?;
        Ok(BookmarkBatch(response.into_inner()).into())
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
        <h1>{ "Hello World" }</h1>
        <Bookmarks/>
        </BounceRoot>
    }
}

#[function_component(Bookmarks)]
fn bookmarks() -> Html {
    let bookmarks = use_query_value::<BookmarkBatch>(Pagination::default().into());
    match bookmarks.result() {
        None => html! {
            <p>{"loading"}</p>
        },
        Some(Ok(b)) => {
            b.0.bookmarks
                .iter()
                .map(|b| html! {<Bookmark bookmark={b.clone()} />})
                .collect::<Html>()
        }
        Some(Err(e)) => html! {
            <h1>{e.to_string()}</h1>
        },
    }
}

#[derive(Properties, PartialEq)]
struct BookmarkProps {
    bookmark: AnnotatedBookmark,
}

#[function_component(Bookmark)]
fn bookmark(BookmarkProps { bookmark }: &BookmarkProps) -> Html {
    html! {
        <article><h2>{&bookmark.bookmark.title}</h2></article>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
