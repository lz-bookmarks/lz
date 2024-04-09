use super::Tag;
use dioxus::prelude::*;
use lz_openapi::types::AnnotatedBookmark;

#[component]
pub(crate) fn Bookmark(abm: AnnotatedBookmark) -> Element {
    rsx! {
        article {
            span { "{abm.bookmark.created_at} " }
            Link { to: "{abm.bookmark.url}", "{abm.bookmark.title}" }
            for tag in abm.tags {
                Tag { name: tag.name }
                "  "
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
