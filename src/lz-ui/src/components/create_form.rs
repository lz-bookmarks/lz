use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{
    use_mutation, use_query_value, Mutation, MutationResult, MutationState, Query, QueryResult,
};
use bounce::BounceStates;
use chrono::Utc;
use lz_openapi::types::{
    BookmarkCreateRequest, CreateBookmarkResponse, Metadata, NewBookmark, NoId,
};
use patternfly_yew::prelude::*;
use url::Url;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::GoddamnIt;

use super::{ModalState, TagSelect};

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
    let url = use_state_eq(|| String::new());
    let onchange = use_callback(url.clone(), |new_value, url| url.set(new_value));
    let valid = use_state(|| true);
    let onvalidated = use_callback(valid.clone(), |state, valid| {
        valid.set(match state {
            InputState::Default | InputState::Success => true,
            _ => false,
        })
    });

    let inner = match &*state {
        &State::EnteringUrl => {
            let onsubmit = Callback::from(move |ev: SubmitEvent| {
                ev.prevent_default();
                if *valid {
                    state.set(State::EnteringData);
                }
            });
            let validator = Validator::from(|ctx: ValidationContext<String>| {
                if ctx.initial {
                    ValidationResult::default()
                } else if ctx.value.is_empty() {
                    ValidationResult::error("URL is required")
                } else if let Err(e) = Url::parse(&ctx.value) {
                    ValidationResult::error(format!("URL is invalid: {}", e))
                } else {
                    ValidationResult::default()
                }
            });
            html! {
                <Form {onvalidated} {onsubmit}>
                    <FormGroupValidated<TextInput> required=true label="URL" {validator}>
                        <TextInput
                            autocomplete={Some("false")}
                            autofocus=true
                            required=true
                            placeholder="URL"
                            id="bookmark_url"
                            value={url.to_string()}
                            {onchange}
                        />
                    </FormGroupValidated<TextInput>>
                </Form>
            }
        }
        &State::EnteringData => html! {
            <FillBookmark onclose={onclose.clone()} url={Url::parse(&*url).unwrap()} />
        },
    };
    html! {
        <>
            <Bullseye plain=true>
                <Modal
                    disable_close_click_outside=true
                    disable_close_escape=true
                    title="Add bookmark"
                    variant={ModalVariant::Medium}
                    onclose={let onclose = onclose.clone(); move |_ev| onclose.emit(())}
                >
                    { inner }
                </Modal>
            </Bullseye>
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
    let valid = use_state(|| false);
    let onvalidated = use_callback(valid.clone(), move |state, valid| {
        valid.set(match state {
            InputState::Default | InputState::Success => true,
            _ => false,
        })
    });
    let tags = use_state(|| vec![]);
    let title = use_state_eq(|| String::default());
    let set_title = use_callback(title.clone(), |new_title, title| title.set(new_title));
    let description = use_state_eq(|| String::default());
    let set_description = use_callback(description.clone(), |new_desc, desc| desc.set(new_desc));
    let notes = use_state_eq(|| String::default());
    let set_notes = use_callback(notes.clone(), |new_notes, notes| notes.set(new_notes));
    let metadata_query = use_query_value::<SaveBookmarkQuery>(Rc::new(url.clone()));
    {
        let res = metadata_query.result().map(|x| x.clone());
        let title_set = title.setter();
        let description_set = description.setter();
        let valid_set = valid.setter();
        use_effect_with(res, move |res| match res {
            Some(Ok(metadata)) => {
                let metadata = metadata.clone();
                if let Some(desc) = &metadata.0.description {
                    description_set.set(desc.to_string());
                }
                title_set.set(metadata.0.title.to_string());
                if !metadata.0.title.is_empty() {
                    valid_set.set(true);
                }
            }
            _ => {}
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
        let valid = valid.clone();
        Callback::from(move |ev: SubmitEvent| {
            let save_bookmark = save_bookmark.clone();
            let tags = tags.clone();
            let description = description.clone();
            let notes = notes.clone();
            let title = title.clone();
            let url = url.clone();
            let created_at = Utc::now();
            let onclose = onclose.clone();

            ev.prevent_default();
            if !*valid {
                return;
            }
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
                // TODO: error-handle & close only when creation went through.
                onclose.emit(());
            })
        })
    };

    match metadata_query.result() {
        Some(_) => html! {
            <Form {onvalidated} onsubmit={save}>
                <TitleInput onchange={set_title} value={(*title).clone()} />
                <FormGroup label="Description">
                    <TextArea onchange={set_description} value={(*description).clone()} />
                </FormGroup>
                <FormGroup label="Notes">
                    <TextArea onchange={set_notes} value={(*notes).clone()} />
                </FormGroup>
                <div class="grid grid-cols-1 gap-1">
                    <label class="font-medium" for="bookmark_tags">{ "Tags" }</label>
                    <TagSelect on_change={Callback::from(move |new_tags| {tags.set(new_tags)})} />
                </div>
                <ActionGroup>
                    <Button
                        loading={save_bookmark.state() == MutationState::Loading}
                        variant={ButtonVariant::Primary}
                        r#type={ButtonType::Submit}
                        disabled={!*valid}
                        label="Save"
                    />
                </ActionGroup>
            </Form>
        },
        None => html! { <div class="skeleton w-full h-full" /> },
    }
}

#[derive(Properties, PartialEq)]
struct TitleInputProps {
    onchange: Callback<String>,
    value: String,
}

// Workaround for https://github.com/patternfly-yew/patternfly-yew/issues/145:
#[function_component(TitleInput)]
fn title_input(TitleInputProps { onchange, value }: &TitleInputProps) -> Html {
    let validator = Validator::from(|ctx: ValidationContext<String>| {
        if ctx.initial {
            ValidationResult::ok()
        } else if ctx.value.is_empty() {
            ValidationResult::error("Title is required")
        } else {
            ValidationResult::ok()
        }
    });
    html! {
        <FormGroupValidated<TextInput> required=true label="Title" {validator}>
            <TextInput required=true autofocus=true {onchange} value={value.clone()} />
        </FormGroupValidated<TextInput>>
    }
}
