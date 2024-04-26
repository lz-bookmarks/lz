use bounce::prelude::*;
use patternfly_yew::prelude::*;
use popper_rs::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::prelude::use_async;
use yew_hooks::{use_click_away, use_event_with_window};

use crate::dispatch_callback;

const ID_SEARCH_ELEMENT: &str = "search-input";

#[derive(PartialEq, Eq, Debug)]
enum TagSelectAction {
    /// Select a choice
    SelectItem(String),

    /// Provide autocomplete options
    FillAutocomplete {
        possibilities: Vec<String>,
    },

    /// Close the autocomplete menu
    CloseAutocomplete,

    /// Reset the entire state to defaults.
    Reset,

    /// Entered text changed.
    TextChange {
        input_value: String,
        position: usize,
    },
    AcceptHint,
}

#[derive(PartialEq, Default, Slice, Clone, Debug)]
pub(self) struct TagSelectState {
    /// The verbatim text field value
    input_value: String,

    /// The position of the insertion point in the verbatim text field
    position: usize,

    /// Possible completions for the field value at the insertion point.
    possibilities: Vec<String>,

    /// Whether to open the autocomplete possibilities box.
    autocomplete_open: bool,

    /// If only one option remains, an entirely auto-completable hint.
    hint: Option<String>,
}

impl Reducible for TagSelectState {
    type Action = TagSelectAction;

    #[tracing::instrument(level = "DEBUG")]
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match &action {
            TagSelectAction::SelectItem(item) => {
                let input_value = self.input_with_suggestion(item.as_str());
                Self {
                    position: input_value.len(),
                    input_value,
                    ..Default::default()
                }
                .into()
            }
            TagSelectAction::FillAutocomplete { possibilities } => {
                let hint = match possibilities.as_slice() {
                    [hintable]
                        if self.is_completing_last_word()
                            && hintable.starts_with(self.incomplete()) =>
                    {
                        Some(self.input_with_suggestion(hintable))
                    }
                    _ => None,
                };
                Self {
                    autocomplete_open: possibilities.len() > 0,
                    hint,
                    possibilities: possibilities.clone(),
                    ..(*self).clone()
                }
                .into()
            }
            TagSelectAction::CloseAutocomplete => Self {
                autocomplete_open: false,
                ..(*self).clone()
            }
            .into(),
            TagSelectAction::Reset => Default::default(),
            TagSelectAction::TextChange {
                input_value,
                position,
            } => Self {
                input_value: input_value.clone(),
                position: *position,
                ..Default::default()
            }
            .into(),
            TagSelectAction::AcceptHint => {
                let input_value = self.hint.as_deref().unwrap_or_default().to_string() + " ";
                Self {
                    position: input_value.len(),
                    input_value,
                    ..Default::default()
                }
                .into()
            }
        }
    }
}

impl TagSelectState {
    fn current_word_positions(input_value: &str, position: usize) -> (usize, usize) {
        if position > input_value.len() {
            tracing::error!(?position, actual_len=?input_value.len(), ?input_value, "Position past end");
            return (0, 0);
        }
        let (before, after) = input_value.split_at(position);
        let end_position = position
            + after
                .chars()
                .position(char::is_whitespace)
                .unwrap_or_else(|| after.len());
        let before_chars = before.chars().collect::<Vec<char>>();
        let start_position = if let Some(whitespace_before) =
            before_chars.into_iter().rposition(char::is_whitespace)
        {
            (whitespace_before + 1).min(end_position)
        } else {
            0
        };
        (start_position, end_position)
    }

    fn is_completing_last_word(&self) -> bool {
        let (_, end_position) = Self::current_word_positions(&self.input_value, self.position);
        end_position == self.input_value.len()
    }

    fn incomplete(&self) -> &str {
        let (start_position, end_position) =
            Self::current_word_positions(&self.input_value, self.position);
        &self.input_value[start_position..end_position]
    }

    /// Return the input field value if the suggested completion has been accepted.
    fn input_with_suggestion(&self, suggestion: &str) -> String {
        let (start_position, end_position) =
            Self::current_word_positions(&self.input_value, self.position);
        let before = &self.input_value[0..start_position];
        let after = &self.input_value[end_position..];
        format!("{before}{suggestion}{after}")
    }

    fn tags(&self) -> Vec<String> {
        self.input_value
            .split_whitespace()
            .map(String::from)
            .collect()
    }
}

#[derive(PartialEq, Debug)]
struct SelectedTags(Vec<String>);

impl Selector for SelectedTags {
    fn select(states: &BounceStates) -> std::rc::Rc<Self> {
        let state = states.get_slice_value::<TagSelectState>();
        SelectedTags(state.tags()).into()
    }
}

#[derive(Properties, Clone, PartialEq, Default)]
pub struct TagSelectProps {
    pub on_change: Callback<Vec<String>>,
}

#[function_component(TagSelect)]
pub fn tag_auto_complete(TagSelectProps { on_change }: &TagSelectProps) -> Html {
    let state = use_slice::<TagSelectState>();
    let tags = use_selector_value::<SelectedTags>();
    // the user provided value
    // clear the value
    let onclear = dispatch_callback(&state, |_| TagSelectAction::Reset);
    // the values filtered by the search value
    let choices = use_async({
        let state = state.clone();
        async move {
            let incomplete = state.incomplete();
            let loc = web_sys::window().unwrap().location();
            let base_url = format!(
                "{}//{}/api/v1",
                loc.protocol().unwrap(),
                loc.host().unwrap()
            );
            let client = lz_openapi::Client::new(&base_url);
            match client.complete_tag().tag_fragment(incomplete).send().await {
                Ok(results) => {
                    state.dispatch(TagSelectAction::FillAutocomplete {
                        possibilities: (*results)
                            .iter()
                            .map(|tag| tag.name.to_string())
                            .collect::<Vec<String>>(),
                    });
                    Ok(())
                }
                Err(_e) => Err(()),
            }
        }
    });

    // references to nodes
    let away_ref = use_node_ref();
    let input_ref = use_node_ref();
    let menu_ref = use_node_ref();

    // popper state
    let popper_state = use_state_eq(popper_rs::prelude::State::default);
    let onstatechange = use_callback(popper_state.clone(), |new_state, state| {
        state.set(new_state)
    });

    // acting on change of the search term data
    let onchange = use_callback((), {
        let state = state.clone();
        let input_ref = input_ref.clone();
        move |input_value: String, ()| {
            let position = input_ref
                .cast::<HtmlInputElement>()
                .map(|elt| {
                    elt.selection_start()
                        .unwrap_or(None)
                        .map(|sel| sel as usize)
                })
                .flatten()
                .unwrap_or_else(|| input_value.len());
            state.dispatch(TagSelectAction::TextChange {
                input_value,
                position,
            });
        }
    });
    // Bubble the entered tags up:
    use_effect_with(tags.clone(), {
        let on_change = on_change.clone();
        move |tags| on_change.emit(tags.0.to_owned())
    });

    // Kick off a search for potential choices:
    use_effect_with(state.incomplete().to_string(), {
        let choices = choices.clone();
        move |incomplete| {
            if incomplete.len() > 2 {
                choices.run();
            }
        }
    });

    // keyboard handling, on top of the menu
    {
        let state = state.clone();
        let input_ref = input_ref.clone();
        let menu_ref = menu_ref.clone();
        use_event_with_window("keydown", move |e: KeyboardEvent| {
            let in_input = input_ref.get().as_deref() == e.target().as_ref();

            match e.key().as_str() {
                "Tab" | "ArrowRight" if in_input => {
                    // if we have a hint (single remaining value)
                    if state.hint.is_some() {
                        if state.autocomplete_open {
                            e.prevent_default();
                        }
                        // set the value
                        state.dispatch(TagSelectAction::AcceptHint);
                        // focus back on the input
                        input_ref.focus();
                    }
                }
                "ArrowUp" | "ArrowDown" if in_input => {
                    // start the menu navigation, the menu component will pick it up from here
                    if let Some(first) = menu_ref
                        .cast::<web_sys::HtmlElement>()
                        .and_then(|ele| {
                            ele.query_selector("li > button:not(:disabled)")
                                .ok()
                                .flatten()
                        })
                        .and_then(|ele| ele.dyn_into::<web_sys::HtmlElement>().ok())
                    {
                        let _ = first.focus();
                    }
                    e.prevent_default();
                }
                "Escape" => {
                    // escape should always close the menu
                    state.dispatch(TagSelectAction::CloseAutocomplete);
                    // focus back on the input control
                    input_ref.focus();
                }
                _ => {}
            }
        });
    }

    // the autocomplete menu
    let autocomplete = {
        let state = state.clone();
        let choices = choices.clone();
        html!(
            if choices.loading {
                <div />
            } else if !state.possibilities.is_empty() {
                <Menu
                    r#ref={menu_ref.clone()}
                    style={&popper_state
                    .styles.popper
                    .extend_with("z-index", "1000")}
                >
                    { for state.possibilities.iter().map(|value| {
                        let onclick = {
                            let state = state.clone();
                            let value = value.to_string();
                            let input_ref = input_ref.clone();
                            Callback::from(move |_| {
                                state.dispatch(TagSelectAction::SelectItem(value.clone()));
                                input_ref.focus();
                            })
                        };
                        html_nested!(
                            <MenuAction {onclick}>{ value }</MenuAction>
                        )
                    }) }
                </Menu>
            }
        )
    };

    {
        // when the user clicks outside the auto-complete menu, we close it
        let close_autocomplete = dispatch_callback(&state, |_| TagSelectAction::CloseAutocomplete);
        use_click_away(away_ref.clone(), move |_| close_autocomplete.emit(()));
    }

    html! {
        <>
            <div ref={away_ref} style="display: block;">
                <SearchInput
                    id={ID_SEARCH_ELEMENT}
                    inner_ref={input_ref.clone()}
                    placeholder="Tags"
                    value={state.input_value.clone()}
                    {onchange}
                    {onclear}
                    hint={state.hint.clone().map(AttrValue::from)}
                />
                <PortalToPopper
                    popper={yew::props!(PopperProperties {
                        target: input_ref.clone(),
                        content: menu_ref.clone(),
                        placement: Placement::Bottom,
                        visible: state.autocomplete_open,
                        modifiers: vec![
                            Modifier::SameWidth(Default::default()),
                        ],
                        onstatechange
                    })}
                    append_to={gloo_utils::document().get_element_by_id(ID_SEARCH_ELEMENT)}
                >
                    { autocomplete }
                </PortalToPopper>
            </div>
        </>
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;
    use tracing_test::traced_test;

    #[traced_test]
    #[test_case("", "", "", &[]; "no entry")]
    #[test_case("foo bar", "", "bar", &["foo", "bar"]; "at the end of a word")]
    #[test_case("foo bar ", "", "", &["foo", "bar"]; "at the end, no new word")]
    #[test_case("foo", " bar", "foo", &["foo", "bar"]; "in the middle")]
    #[test_case("foo ", " bar", "", &["foo", "bar"]; "between words")]
    fn incompleteness_at_positions(before: &str, after: &str, incomplete: &str, tags: &[&str]) {
        let input_value = format!("{before}{after}");
        let state = TagSelectState {
            input_value,
            position: before.len(),
            ..Default::default()
        };
        assert_eq!(state.incomplete(), incomplete, "incomplete string");
        assert_eq!(state.tags(), tags, "tags");
    }

    #[traced_test]
    #[test_case("", "", "", ""; "no entry")]
    #[test_case("foo bar", "", "barium", "foo barium"; "at the end of a word")]
    #[test_case("foo bar ", "", "test", "foo bar test"; "at the end, new word")]
    #[test_case("foo", " bar", "foo-fighters", "foo-fighters bar"; "in the middle")]
    #[test_case("foo ", "bar", "foo-fighters", "foo foo-fighters"; "middle of second word")]
    #[test_case("foo ", " bar", "night", "foo night bar"; "between words")]
    fn acceptance_at_positions(before: &str, after: &str, suggestion: &str, result: &str) {
        let input_value = format!("{before}{after}");
        let state = TagSelectState {
            input_value,
            position: before.len(),
            ..Default::default()
        };
        assert_eq!(state.input_with_suggestion(suggestion), result,);
    }
}
