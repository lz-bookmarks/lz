use lz_openapi::types::AnnotatedBookmark;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BookmarkProps {
    pub bookmark: AnnotatedBookmark,
}

#[function_component(Bookmark)]
pub fn bookmark(BookmarkProps { bookmark }: &BookmarkProps) -> Html {
    html! {
        <article key={bookmark.bookmark.id.to_string()}>
        <h2><a href={bookmark.bookmark.url.to_string()}>{&bookmark.bookmark.title}</a></h2>
        </article>
    }
}
