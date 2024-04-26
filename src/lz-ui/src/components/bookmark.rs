use crate::route::*;
use lz_openapi::types::AnnotatedBookmark;
use patternfly_yew::prelude::*;
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
                <Link<Route> to={Route::SearchTag{tag: tag.name.clone()}}>
                    <Chip text={tag.name.clone()} />
                </Link<Route>>
            }
        })
        .collect::<Html>();
    let description = bookmark
        .bookmark
        .description
        .as_ref()
        .map(|d| html! { <p>{ d }</p> })
        .unwrap_or_else(|| html! {});
    let notes = bookmark
        .bookmark
        .notes
        .as_ref()
        .map(|n| html! { <blockquote>{ n }</blockquote> })
        .unwrap_or_else(|| html! {});
    html! {
        <Card key={bookmark.bookmark.id.to_string()} size={CardSize::Compact}>
            <CardHeader
                actions={yew::props!(CardHeaderActionsObject {
            actions: tags,
        })}
            >
                <a href={bookmark.bookmark.url.to_string()} target="_new">
                    <Button
                        variant={ButtonVariant::InlineLink}
                        label={bookmark.bookmark.title.clone()}
                    />
                </a>
            </CardHeader>
            <CardBody>
                <Content>{ description }{ notes }</Content>
            </CardBody>
        </Card>
    }
}
