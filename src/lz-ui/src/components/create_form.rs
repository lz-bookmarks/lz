use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::BounceStates;
use lz_openapi::types::Metadata;
use url::Url;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::GoddamnIt;

use super::{ModalState, TagSelect};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onclose: Callback<MouseEvent>,
}

#[function_component(CreateForm)]
pub fn create_form(Props { onclose }: &Props) -> Html {
    let state = use_context::<ModalState>().expect("context needs to be provided");
    html! {
        if state == ModalState::CreateBookmark {
            <VisibleCreateForm {onclose} />
        } else {
            <></>
        }
    }
}

#[derive(Properties, PartialEq)]
struct VisibleProps {
    onclose: Callback<MouseEvent>,
}

#[derive(Clone, PartialEq, Debug)]
struct BookmarkData {
    url: String,
    title: String,
    description: String,
    tags: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    EnteringUrl,
    EnteringData,
}

#[derive(PartialEq, Debug, Clone)]
struct SaveBookmarkQuery(Metadata);

#[async_trait(?Send)]
impl Query for SaveBookmarkQuery {
    type Input = Url;

    type Error = GoddamnIt;

    async fn query(_states: &BounceStates, input: Rc<Url>) -> QueryResult<Self> {
        let loc = web_sys::window().unwrap().location();
        let base_url = format!(
            "{}//{}/api/v1",
            loc.protocol().unwrap(),
            loc.host().unwrap()
        );

        let client = lz_openapi::Client::new(&base_url);
        let res = client
            .fetch_page_metadata()
            .url(input.to_string())
            .send()
            .await
            .map_err(GoddamnIt::new)?;
        let md = res.into_inner();
        Ok(SaveBookmarkQuery(Metadata {
            title: md.title,
            description: md.description,
        })
        .into())
    }
}

#[function_component(VisibleCreateForm)]
fn visible_create_form(VisibleProps { onclose }: &VisibleProps) -> Html {
    let state = use_state(|| State::EnteringUrl);
    let url = use_state(|| Url::parse("").ok());

    let inner = match &*state {
        &State::EnteringUrl => {
            let onsubmit = Callback::from(move |_e| {
                state.set(State::EnteringData);
            });
            html! {
                <div class="join place-self-center">
                    <form {onsubmit}>
                        <input
                            type="url"
                            class={classes!("input", "input-bordered", "join-item", "invalid:border-red-600")}
                            placeholder="URL"
                            id="bookmark_url"
                            oninput={let url = url.clone();
                            move |e: InputEvent| {
                                let input = e.target_dyn_into::<HtmlInputElement>().unwrap();
                                if let Ok(u) = Url::parse(&input.value()) {
                                    url.set(Some(u));
                                }
                            }}
                        />
                        <input
                            type="submit"
                            class={classes!("btn", "join-item")}
                            disabled={url.is_none()}
                            value="Add"
                        />
                    </form>
                </div>
            }
        }
        &State::EnteringData => html! {
            if let Some(url) = &*url {
                <FillBookmark url={url.clone()} />
            } else {
                <p>{ "Error: URL is invalid" }</p>
            }
        },
    };
    html! {
        <>
            <input type="checkbox" id="create_modal_visibility" class="modal-toggle" checked=true />
            <div class="modal" role="dialog">
                <div class={classes!("modal-box", "max-h-none")}>
                    <h3 class="font-bold text-lg">{ "Add bookmark" }</h3>
                    { inner }
                    <div class="modal-action">
                        <button
                            onclick={onclose}
                            class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                        >
                            { "✕" }
                        </button>
                    </div>
                </div>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq, Debug)]
struct FillBookmarkProps {
    url: Url,
}

#[function_component(FillBookmark)]
fn fill_bookmark(FillBookmarkProps { url }: &FillBookmarkProps) -> Html {
    let tags = use_state(|| vec![]);
    let title = use_state(|| String::default());
    let description = use_state(|| String::default());
    let metadata_query = use_query_value::<SaveBookmarkQuery>(Rc::new(url.clone()));
    {
        let res = metadata_query.result().map(|x| x.clone());
        let title_set = title.setter();
        let description_set = description.setter();
        use_effect_with(res, move |res| {
            tracing::info!(?res, "effect");
            match res {
                Some(Ok(metadata)) => {
                    let metadata = metadata.clone();
                    if let Some(desc) = &metadata.0.description {
                        description_set.set(desc.to_string());
                    }
                    title_set.set(metadata.0.title.to_string());
                }
                _ => {}
            }
        });
    }

    match metadata_query.result() {
        Some(_) => html! {
            <div
                class={classes!("grid", "grid-cols-1", "gap-4", "place-content-start", "h-[600px]")}
            >
                <div class="grid grid-cols-1 gap-1">
                    <label class="font-medium" for="bookmark_title">{ "Title" }</label>
                    <input
                        id="bookmark_title"
                        class={classes!("input", "input-bordered", "w-full", "max-w-xs")}
                        type="text"
                        value={(*title).clone()}
                    />
                </div>
                <div class="grid grid-cols-1 gap-1">
                    <label class="font-medium" for="bookmark_description">{ "Description" }</label>
                    <textarea
                        id="bookmark_description"
                        class={classes!("textarea", "textarea-bordered", "w-full", "max-w-xs")}
                        placeholder="Description"
                        value={(*description).clone()}
                    />
                </div>
                <div class="grid grid-cols-1 gap-1">
                    <label class="font-medium" for="bookmark_tags">{ "Tags" }</label>
                    <TagSelect on_change={Callback::from(move |new_tags| {tags.set(new_tags)})} />
                </div>
            </div>
        },
        None => html! { <div class="skeleton w-full h-full" /> },
    }
}
