use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::{BounceRoot, BounceStates};
use lz_openapi::types::builder::ListRequest;
use lz_openapi::types::{
    AnnotatedBookmark, BookmarkId, BookmarkSearch, ListBookmarksMatchingResponse, Pagination,
};
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
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

impl BookmarksProps {
    fn as_body(&self) -> ListRequest {
        ListRequest::default()
            .cursor(self.cursor)
            .query(self.query.clone())
    }
}

#[async_trait(?Send)]
impl Query for BookmarkBatch {
    type Input = BookmarksProps;
    type Error = GoddamnIt;

    async fn query(_states: &BounceStates, input: Rc<BookmarksProps>) -> QueryResult<Self> {
        let base_url = web_sys::window()
            .map(|w| w.location().href().unwrap() + "api/v1")
            .unwrap();

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

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
        <h1>{ "Hello World" }</h1>
        <Bookmarks cursor={None} query={vec![]}/>
        </BounceRoot>
    }
}

#[derive(Properties, Default, PartialEq, Clone, Eq, Hash, Debug)]
struct BookmarksProps {
    cursor: Option<BookmarkId>,
    query: Vec<BookmarkSearch>,
}

#[function_component(Bookmarks)]
fn bookmarks(props: &BookmarksProps) -> Html {
    let load_next = use_state(|| false);
    let bookmarks = use_query_value::<BookmarkBatch>(Rc::new(props.clone()));
    match bookmarks.result() {
        None => html! {
            <p>{"loading..."}</p>
        },
        Some(Ok(b)) => {
            let bookmark_items =
                b.0.bookmarks
                    .iter()
                    .map(|b| html! {<Bookmark bookmark={b.clone()} />})
                    .collect::<Html>();
            html! {
                <section>
                <>{bookmark_items}
                {if let Some(next) = b.next_cursor {
                    if !*load_next {
                        html!{
                            <button onclick={move |_ev| {
                                tracing::info!(?next, "hi");
                                load_next.set(true);
                            }}>{
                                "Load more..."
                            }</button>
                        }
                    } else {
                        html!{
                            <Bookmarks cursor={next} query={props.query.clone()}/>
                        }
                    }
                } else { html!{} }}
                </>
                </section>
            }
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
        <article key={bookmark.bookmark.id.to_string()}>
        <h2><a href={bookmark.bookmark.url.to_string()}>{&bookmark.bookmark.title}</a></h2>
        </article>
    }
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers, see note below
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events

    yew::Renderer::<App>::new().render();
}
