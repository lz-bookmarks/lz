#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
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
    use_effect(|| debug!("hi"));
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let bookmarks = use_resource(move || async move {
        let client = lz_openapi::Client::new("http://localhost:3000/api/v1");
        client.list_bookmarks().send().await
    });

    rsx! {
        Link { to: Route::Blog { id: count() }, "Go to blog" }
        div {
            h1 { "High-Five counter: {count}" }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
            match &*bookmarks.read_unchecked() {
                Some(Ok(bookmarks)) =>
                rsx!{
                    ul {
                        for abm in &bookmarks.bookmarks {
                            li {
                                a {
                                    href: "{abm.bookmark.url}",
                                    "{abm.bookmark.title}"
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => {error!{?e}; None},
                None => rsx!{ p { "Loading..."}},
            }
        }
    }
}
