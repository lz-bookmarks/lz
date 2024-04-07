use dioxus::prelude::*;
use lz_openapi::types::AnnotatedBookmark;

#[component]
pub(crate) fn Bookmark(abm: AnnotatedBookmark) -> Element {
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
            if let Some(notes) = abm.bookmark.notes {
                blockquote { "{notes}" }
            }
        }
    }
}
