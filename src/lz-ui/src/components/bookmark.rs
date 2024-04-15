use crate::route::*;
use lz_openapi::types::AnnotatedBookmark;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BookmarkProps {
    pub bookmark: AnnotatedBookmark,
}

#[function_component(Bookmark)]
pub fn bookmark(BookmarkProps { bookmark }: &BookmarkProps) -> Html {
    let tags = bookmark
        .tags
        .iter()
        .map(|tag| {
            html! {
                <Link<Route>
                    classes={classes!["badge", "badge-ghost"]}
                    to={Route::SearchTag{tag: tag.name.clone()}}
                >
                    { tag.name.clone() }
                </Link<Route>>
            }
        })
        .collect::<Html>();
    let description = bookmark
        .bookmark
        .description
        .as_ref()
        .map(|d| html! { <p class="prose">{ d }</p> })
        .unwrap_or_else(|| html! {});
    let notes = bookmark
        .bookmark
        .notes
        .as_ref()
        .map(|n| html! { <blockquote class="prose">{ n }</blockquote> })
        .unwrap_or_else(|| html! {});
    html! {
        <article
            key={bookmark.bookmark.id.to_string()}
            class="card card-compact hover:card-bordered"
        >
            <div class="card-body">
                <h2 class="card-title">
                    <a
                        class="link link-neutral truncate text-ellipsis"
                        href={bookmark.bookmark.url.to_string()}
                    >
                        { &bookmark.bookmark.title }
                    </a>
                </h2>
                <div class="card-actions">{ tags }</div>
                { description }
                { notes }
            </div>
        </article>
    }
}
