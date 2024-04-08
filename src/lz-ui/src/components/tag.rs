use dioxus::prelude::*;

#[component]
pub fn Tag(name: String) -> Element {
    rsx! {
        a {
            href: "hi",
            "{name}"
        }
    }
}
