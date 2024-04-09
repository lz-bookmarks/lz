use dioxus::prelude::*;

use crate::{use_search_criteria, Route};

#[component]
pub fn Tag(name: String) -> Element {
    let search = use_search_criteria().read().with_tag(&name);

    rsx! {
        Link {
            to: Route::Bookmarks{search: search.to_route()},
            "{name}"
        }
    }
}
