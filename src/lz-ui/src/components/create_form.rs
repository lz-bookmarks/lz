use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_mutation, use_query_value, Mutation, MutationResult, Query, QueryResult};
use bounce::BounceStates;
use chrono::Utc;
use lz_openapi::types::{
    BookmarkCreateRequest, CreateBookmarkResponse, Metadata, NewBookmark, NoId,
};
use url::Url;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::GoddamnIt;

use super::{BookmarkEditText, ModalState, TagSelect};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onclose: Callback<()>,
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
    onclose: Callback<()>,
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

/// Saves a new bookmark.
#[derive(PartialEq, Clone, Debug)]
struct SaveBookmarkMutation(CreateBookmarkResponse);

#[async_trait(?Send)]
impl Mutation for SaveBookmarkMutation {
    type Input = BookmarkCreateRequest;
    type Error = GoddamnIt;

    async fn run(_states: &BounceStates, input: Rc<BookmarkCreateRequest>) -> MutationResult<Self> {
        let loc = web_sys::window().unwrap().location();
        let base_url = format!(
            "{}//{}/api/v1",
            loc.protocol().unwrap(),
            loc.host().unwrap()
        );

        let client = lz_openapi::Client::new(&base_url);
        let result = client
            .create_bookmark()
            .body(&*input)
            .send()
            .await
            .map_err(GoddamnIt::new)?;
        Ok(Rc::new(SaveBookmarkMutation(result.into_inner())))
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
                <FillBookmark onclose={onclose.clone()} url={url.clone()} />
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
                            onclick={let onclose = onclose.clone(); {move |_ev| onclose.emit(())}}
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
    onclose: Callback<()>,
}

#[function_component(FillBookmark)]
fn fill_bookmark(FillBookmarkProps { url, onclose }: &FillBookmarkProps) -> Html {
    let tags = use_state(|| vec![]);
    let title = use_state(|| String::default());
    let title_callback = {
        let setter = title.setter();
        Callback::from(move |x| {
            setter.set(x);
        })
    };
    let description = use_state(|| String::default());
    let description_callback = {
        let setter = description.setter();
        Callback::from(move |x| {
            setter.set(x);
        })
    };
    let notes = use_state(|| String::default());
    let notes_callback = {
        let setter = notes.setter();
        Callback::from(move |x| {
            tracing::info!(?x, "setting notes");
            setter.set(x);
        })
    };
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
    let save_bookmark = use_mutation::<SaveBookmarkMutation>();
    let save = {
        let save_bookmark = save_bookmark.clone();
        let description = description.clone();
        let notes = notes.clone();
        let title = title.clone();
        let url = url.clone();
        let tags = tags.clone();
        let onclose = onclose.clone();
        Callback::from(move |_| {
            let save_bookmark = save_bookmark.clone();
            let tags = tags.clone();
            let description = description.clone();
            let notes = notes.clone();
            let title = title.clone();
            let url = url.clone();
            let created_at = Utc::now();
            let onclose = onclose.clone();
            spawn_local(async move {
                let notes = if *notes == "" {
                    None
                } else {
                    Some(notes.to_string())
                };
                let _ = save_bookmark // TODO: error-handle
                    .run(BookmarkCreateRequest {
                        associations: vec![],
                        tag_names: (*tags).clone(),
                        bookmark: NewBookmark {
                            id: NoId(serde_json::Value::Null),
                            user_id: NoId(serde_json::Value::Null),
                            accessed_at: None,
                            created_at,
                            description: Some((*description).to_string()),
                            modified_at: None,
                            notes,
                            shared: None,
                            title: (*title).to_string(),
                            unread: None,
                            url: url.to_string(),
                            website_description: None, // TODO
                            website_title: None,       // TODO
                        },
                    })
                    .await;
                onclose.emit(());
            })
        })
    };

    match metadata_query.result() {
        Some(_) => html! {
            <>
                <div
                    class={classes!("grid", "grid-cols-1", "gap-4", "place-content-start", "h-[500px]")}
                >
                    <BookmarkEditText
                        name="Title"
                        id="bookmark_title"
                        multiline=false
                        value={(*title).clone()}
                        onchange={title_callback}
                    />
                    <BookmarkEditText
                        name="Description"
                        id="bookmark_description"
                        multiline=true
                        value={(*description).clone()}
                        onchange={description_callback}
                    />
                    <BookmarkEditText
                        name="Notes"
                        id="bookmark_notes"
                        multiline=true
                        value={(*notes).clone()}
                        onchange={notes_callback}
                    />
                    <div class="grid grid-cols-1 gap-1">
                        <label class="font-medium" for="bookmark_tags">{ "Tags" }</label>
                        <TagSelect
                            on_change={Callback::from(move |new_tags| {tags.set(new_tags)})}
                        />
                    </div>
                </div>
                <div class="modal-action">
                    <button class="btn" onclick={save}>{ "Save" }</button>
                </div>
            </>
        },
        None => html! { <div class="skeleton w-full h-full" /> },
    }
}
