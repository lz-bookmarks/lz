#![allow(non_snake_case)]

use dioxus::prelude::*;
use lz_openapi::types::AnnotatedBookmark;
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
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let base_url = web_sys::window()
        .map(|w| w.location().href().unwrap() + "api/v1")
        .unwrap();
    let bookmarks = use_resource(move || {
        let base_url = base_url.clone();
        async move {
            let client = lz_openapi::Client::new(&base_url);
            client.list_bookmarks().send().await
        }
    });

    rsx! {
        div {
            h1 { "My Bookmarks" }
            match &*bookmarks.read_unchecked() {
                Some(Ok(bookmarks)) =>
                rsx!{
                    section {
                        for abm in &bookmarks.bookmarks {
                            Bookmark { abm: abm.clone() }
                        }
                    }
                },
                Some(Err(e)) => {error!{?e}; None},
                None => rsx!{ p { "Loading..."}},
            }
        }
    }
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
