use std::rc::Rc;

use async_trait::async_trait;
use bounce::prelude::*;
use bounce::query::{
    use_mutation, use_query_value, Mutation, MutationResult, MutationState, Query, QueryResult,
};
use chrono::Utc;
use lz_openapi::types::{
    AnnotatedBookmark, BookmarkCreateRequest, CreateBookmarkResponse, NewBookmark, NoId,
    UrlMetadata,
};
use patternfly_yew::prelude::*;
use url::Url;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::{dispatch_callback, GoddamnIt};

use super::{CloseModal, TagSelect};

#[derive(Properties, PartialEq)]
pub struct VisibleProps {
    pub onclose: Callback<()>,
}

#[derive(Clone, Default, PartialEq, Debug, Slice)]
#[bounce(with_notion(CloseModal))]
struct BookmarkData {
    existing_bookmark: Option<AnnotatedBookmark>,
    url: String,
    title: String,
    description: String,
    website_description: Option<String>,
    website_title: Option<String>,
    notes: String,
    tags: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    EnteringUrl,
    EnteringData,
}

enum BookmarkAction {
    SetUrl(String),
    SetTitle(String),
    SetDescription(String),
    SetNotes(String),
    SetTags(Vec<String>),
    FromMetadata(UrlMetadata),
}

impl Reducible for BookmarkData {
    type Action = BookmarkAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            BookmarkAction::SetUrl(url) => Self {
                url,
                ..(*self).clone()
            }
            .into(),
            BookmarkAction::SetTitle(title) => Self {
                title,
                ..(*self).clone()
            }
            .into(),
            BookmarkAction::SetDescription(description) => Self {
                description,
                ..(*self).clone()
            }
            .into(),
            BookmarkAction::SetNotes(notes) => Self {
                notes,
                ..(*self).clone()
            }
            .into(),

            BookmarkAction::SetTags(tags) => Self {
                tags,
                ..(*self).clone()
            }
            .into(),
            BookmarkAction::FromMetadata(UrlMetadata {
                title,
                description,
                existing_bookmark,
            }) => {
                let website_title = (title != "").then(|| title.clone());
                let website_description = description.clone();
                let description = description.unwrap_or("".to_string());
                Self {
                    website_title,
                    website_description,
                    title,
                    description,
                    existing_bookmark,
                    ..(*self).clone()
                }
                .into()
            }
        }
    }
}

impl WithNotion<CloseModal> for BookmarkData {
    fn apply(self: std::rc::Rc<Self>, _notion: std::rc::Rc<CloseModal>) -> std::rc::Rc<Self> {
        Default::default()
    }
}

impl BookmarkData {
    fn to_create_request(&self) -> BookmarkCreateRequest {
        let created_at = Utc::now();
        BookmarkCreateRequest {
            associations: vec![],
            bookmark: NewBookmark {
                id: NoId(serde_json::Value::Null),
                user_id: NoId(serde_json::Value::Null),
                accessed_at: None,
                created_at,
                description: Some(self.description.to_string()),
                modified_at: None,
                notes: if self.notes.is_empty() {
                    None
                } else {
                    Some(self.notes.to_string())
                },
                shared: None,
                title: self.title.to_string(),
                unread: None,
                url: self.url.to_string(),
                website_description: self.website_description.clone(),
                website_title: self.website_title.clone(),
            },
            tag_names: self.tags.clone(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
struct SaveBookmarkQuery(UrlMetadata);

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
        Ok(SaveBookmarkQuery(UrlMetadata {
            title: md.title,
            description: md.description,
            existing_bookmark: md.existing_bookmark,
        })
        .into())
    }
}

/// Saves a new bookmark.
#[derive(PartialEq, Clone, Debug)]
struct SaveBookmarkMutation(CreateBookmarkResponse);

#[async_trait(?Send)]
impl Mutation for SaveBookmarkMutation {
    type Input = ();
    type Error = GoddamnIt;

    async fn run(states: &BounceStates, _input: Rc<()>) -> MutationResult<Self> {
        let bookmark_data = states.get_slice_value::<BookmarkData>();
        let loc = web_sys::window().unwrap().location();
        let base_url = format!(
            "{}//{}/api/v1",
            loc.protocol().unwrap(),
            loc.host().unwrap()
        );

        let client = lz_openapi::Client::new(&base_url);
        let result = client
            .create_bookmark()
            .body(bookmark_data.to_create_request())
            .send()
            .await
            .map_err(GoddamnIt::new)?;
        Ok(Rc::new(SaveBookmarkMutation(result.into_inner())))
    }
}

#[function_component(CreateForm)]
pub fn create_form(VisibleProps { onclose }: &VisibleProps) -> Html {
    let state = use_state(|| State::EnteringUrl);
    let bookmark_data = use_slice::<BookmarkData>();
    let onchange = dispatch_callback(&bookmark_data, BookmarkAction::SetUrl);
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
                            value={bookmark_data.url.to_string()}
                            {onchange}
                        />
                    </FormGroupValidated<TextInput>>
                </Form>
            }
        }
        &State::EnteringData => html! {
            <FillBookmark onclose={onclose.clone()} url={Url::parse(&bookmark_data.url).unwrap()} />
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
    let bookmark_data = use_slice::<BookmarkData>();
    let set_title = dispatch_callback(&bookmark_data, BookmarkAction::SetTitle);
    let set_description = dispatch_callback(&bookmark_data, BookmarkAction::SetDescription);
    let set_tags = dispatch_callback(&bookmark_data, BookmarkAction::SetTags);
    let set_notes = dispatch_callback(&bookmark_data, BookmarkAction::SetNotes);
    let metadata_query = use_query_value::<SaveBookmarkQuery>(Rc::new(url.clone()));
    {
        let res = metadata_query.result().map(|x| x.clone());
        let valid_set = valid.setter();
        let bookmark_data = bookmark_data.clone();
        use_effect_with(res, move |res| match res {
            Some(Ok(metadata)) => {
                if !metadata.0.title.is_empty() {
                    valid_set.set(true);
                }
                bookmark_data.dispatch(BookmarkAction::FromMetadata(metadata.0.clone()));
            }
            _ => valid_set.set(false),
        });
    }
    let save_bookmark = use_mutation::<SaveBookmarkMutation>();
    let save = {
        let save_bookmark = save_bookmark.clone();
        let onclose = onclose.clone();
        let valid = valid.clone();
        Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();
            if !*valid {
                return;
            }
            let onclose = onclose.clone();
            let save_bookmark = save_bookmark.clone();
            spawn_local(async move {
                let _ = save_bookmark.run(()).await;
                // TODO: error-handle & close only when creation went through.
                onclose.emit(());
            })
        })
    };

    match metadata_query.result() {
        Some(_) => html! {
            <Form {onvalidated} onsubmit={save}>
                <FormGroup label="URL">
                    <TextInput value={url.to_string()} disabled=true />
                </FormGroup>
                <TitleInput onchange={set_title} value={bookmark_data.title.clone()} />
                <FormGroup label="Description">
                    <TextArea onchange={set_description} value={bookmark_data.description.clone()} />
                </FormGroup>
                <FormGroup label="Notes">
                    <TextArea onchange={set_notes} value={bookmark_data.notes.clone()} />
                </FormGroup>
                <FormGroup label="Tags">
                    <TagSelect on_change={set_tags} />
                </FormGroup>
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
            <TextInput required=true {onchange} value={value.clone()} />
        </FormGroupValidated<TextInput>>
    }
}
