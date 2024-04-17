use lz_openapi::types::BookmarkSearch;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/tag/:tag")]
    SearchTag { tag: String },

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    let main = match routes {
        Route::Home => html! { <Bookmarks cursor={None} query={vec![]} /> },
        Route::SearchTag { tag } => {
            html! { <Bookmarks cursor={None} query={vec![BookmarkSearch::Tag(tag.into())]} /> }
        }
        Route::NotFound => html! { <h1>{ "404, not found" }</h1> },
    };
    html! {
        <Layout>
            <div class="flex">
                <section class="container flex-auto mx-auto">{ main }</section>
            </div>
        </Layout>
    }
}
