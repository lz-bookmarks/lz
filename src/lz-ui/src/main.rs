#![allow(non_snake_case)]

use std::ops::Deref;

use dioxus::prelude::*;
use tracing::Level;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use components::BookmarkList;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    BookmarkList {},
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
