use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_query_value, Query, QueryResult};
use bounce::BounceStates;
use lz_http::Metadata;
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
        let res = lz_http::lookup_page_from_web(&*input)
            .await
            .map_err(GoddamnIt::new)?;
        Ok(SaveBookmarkQuery(res).into())
    }
}

#[function_component(VisibleCreateForm)]
fn visible_create_form(VisibleProps { onclose }: &VisibleProps) -> Html {
    let state = use_state(|| State::EnteringUrl);
    let url = use_state(|| Url::parse("").ok());

    let inner = match &*state {
        &State::EnteringUrl => {
            let onclick = Callback::from(move |_e| {
                state.set(State::EnteringData);
            });
            html! {
                <div class="join">
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
                    <button
                        class={classes!("btn", "join-item")}
                        disabled={url.is_none()}
                        {onclick}
                    >
                        { "Add" }
                    </button>
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
                <div class="modal-box">
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
        let title = title.clone();
        use_effect_with(res, move |res| {
            tracing::info!(?res, "effect");
            match res {
                Some(Ok(metadata)) => {
                    let metadata = metadata.clone();
                    title.set(metadata.0.title.to_string());
                    if let Some(desc) = &metadata.0.description {
                        tracing::info!(?desc, "setting stuff");
                        description.set(desc.to_string());
                    }
                }
                _ => {}
            }
        });
    }

    html! {
        <>
            <input type="text" value={(*title).clone()} />
            <TagSelect on_change={Callback::from(move |new_tags| {tags.set(new_tags)})} />
        </>
    }
}
