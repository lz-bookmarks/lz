use itertools::Itertools as _;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_autocomplete::view::{Context, RenderHtml};
use yew_autocomplete::{Autocomplete, ItemResolver, ItemResolverResult};

#[derive(Properties, Clone, PartialEq, Default)]
pub struct TagSelectProps {
    pub on_change: Callback<Vec<String>>,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Inactive,
    Active { initial: String, index: usize },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Msg {
    Activated(usize),
    Selected(Vec<String>),
    Deleted,
}

pub struct TagSelect {
    state: State,
    tags: Vec<String>,
    on_change: Callback<Vec<String>>,
    input_field_ref: NodeRef,
}

impl Component for TagSelect {
    type Message = Msg;

    type Properties = TagSelectProps;

    fn create(ctx: &yew::prelude::Context<Self>) -> Self {
        TagSelect {
            state: State::Inactive,
            tags: vec![],
            on_change: ctx.props().on_change.clone(),
            input_field_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> Html {
        let mut tag_pills = vec![];
        for (index, tag) in self.tags.iter().enumerate() {
            let onchange = match self.state {
                State::Inactive => Callback::from(|selection| {
                    tracing::error!(?selection, "Impossible: selection was made while inactive")
                }),
                State::Active { .. } => ctx.link().callback(Msg::Selected),
            };
            let onclick = match self.state {
                State::Inactive => ctx.link().callback(move |_ev| Msg::Activated(index)),
                State::Active { .. } => Callback::from(|_ev| ()),
            };
            let pill_state = match (index, &self.state) {
                (i, State::Active { index, .. }) if i == *index => {
                    SingleState::Active(self.input_field_ref.clone())
                }
                (_, State::Inactive) | (_, State::Active { .. }) => {
                    SingleState::Selected(tag.clone())
                }
            };
            tag_pills.push(html! {
                <TagSelectSinglePill
                    state={pill_state}
                    {onclick}
                    {onchange}
                    oncancel={ctx.link().callback(|_| Msg::Deleted)}
                />
            });
        }
        let self_onclick = if self.state == State::Inactive {
            let index = self.tags.len();
            ctx.link().callback(move |_| Msg::Activated(index))
        } else {
            Callback::from(|_| {})
        };
        html! {
            <div
                class={classes!("flex", "flex-wrap", "w-full", "h-5", "justify-start", "border-2", "border-solid")}
            >
                { tag_pills }
                <button onclick={self_onclick} class={classes!("flex-1")}>{ " " }</button>
            </div>
        }
    }

    fn update(&mut self, _ctx: &yew::prelude::Context<Self>, msg: Self::Message) -> bool {
        match (msg, &self.state) {
            (Msg::Deleted, State::Active { index, .. }) => {
                tracing::trace!(?index, "deleted");
                self.tags.remove(*index);
                self.state = State::Inactive;
                true
            }
            (Msg::Activated(index), State::Inactive) => {
                tracing::trace!(?index, "activated!");
                if index >= self.tags.len() {
                    self.tags.push("".to_string());
                }
                self.state = State::Active {
                    index,
                    initial: self.tags[index].clone(),
                };
                true
            }
            (Msg::Selected(mut selection), State::Active { index, .. }) => {
                self.tags[*index] = selection.remove(0);
                self.state = State::Inactive;
                self.input_field_ref = Default::default();
                self.on_change.emit(self.tags.clone());
                tracing::trace!(?selection, tags=?self.tags, "selected");
                true
            }
            (x, y) => unreachable!(
                "Reached state ({:?}, {:?}), which should not be reachable",
                x, y
            ),
        }
    }

    fn rendered(&mut self, _ctx: &yew::prelude::Context<Self>, _first_render: bool) {
        if let (State::Active { initial, .. }, Some(input_field)) =
            (&self.state, self.input_field_ref.cast::<HtmlInputElement>())
        {
            let _ = input_field.focus();
            input_field.set_value(initial.as_str());
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SingleProps {
    pub state: SingleState,
    pub onclick: Callback<MouseEvent>,
    pub onchange: Callback<Vec<String>>,
    pub oncancel: Callback<()>,
}

#[derive(Clone, PartialEq)]
enum SingleState {
    Selected(String),
    Active(NodeRef),
}

#[function_component(TagSelectSinglePill)]
fn tag_select_single_pill(
    SingleProps {
        state,
        onclick,
        onchange,
        oncancel,
    }: &SingleProps,
) -> Html {
    match state {
        SingleState::Selected(tag) => {
            html! {
                <span {onclick} class={classes!("flex-none", "badge", "badge-neutral")}>
                    { tag }
                </span>
            }
        }
        SingleState::Active(node) => {
            let resolve_items: ItemResolver<String> =
                Callback::from(|fragment: String| -> ItemResolverResult<String> {
                    Box::pin(async {
                        let loc = web_sys::window().unwrap().location();
                        let base_url = format!(
                            "{}//{}/api/v1",
                            loc.protocol().unwrap(),
                            loc.host().unwrap()
                        );
                        let client = lz_openapi::Client::new(&base_url);
                        match client.complete_tag().tag_fragment(&fragment).send().await {
                            Ok(results) => Ok((*results)
                                .iter()
                                .map(|tag| tag.name.to_string())
                                // Add the fragment, in case it wasn't found yet:
                                .chain([fragment])
                                .unique()
                                .collect::<Vec<String>>()),
                            Err(_e) => Err(()),
                        }
                    })
                });
            html! {
                <span class={classes!("flex-none", "badge", "badge-neutral")}>
                    <Autocomplete<String>
                        {onchange}
                        {resolve_items}
                        multi_select=false
                        auto=true
                        show_selected=false
                    >
                        <Tailwind {node} {oncancel} />
                    </Autocomplete<String>>
                </span>
            }
        }
    }
}

#[derive(Properties, PartialEq)]
struct TailwindProps {
    node: NodeRef,
    oncancel: Callback<()>,
}

#[function_component(Tailwind)]
fn tailwind(TailwindProps { node, oncancel }: &TailwindProps) -> Html {
    let view_ctx = use_context::<Context<String>>().expect("view::Context wasn't provided");
    let oncancel = oncancel.clone();
    let onkeydown = Callback::from(move |ev: KeyboardEvent| {
        let input = ev.target_dyn_into::<HtmlInputElement>().unwrap();
        if input.value() == "" && ev.which() == 8 {
            oncancel.emit(());
        }
    });
    let input_cb = view_ctx.callbacks.on_input.clone();
    let oninput = move |e: InputEvent| {
        let input = e.target_dyn_into::<HtmlInputElement>().unwrap();
        let value = input.value();
        tracing::trace!(key = ?e, "key pressed");
        input_cb.emit(value);
    };
    let select_item = view_ctx.callbacks.select_item.clone();
    let options = view_ctx
        .items
        .iter()
        .enumerate()
        .map(move |(index, item)| {
            let select_item = select_item.clone();
            html! {
                <li>
                    <a
                        class={classes!("hover:bg-gray-300", (Some(index) == view_ctx.highlighted).then_some("bg-gray-300"))}
                        onclick={move |e: MouseEvent| {
                    e.prevent_default();
                    select_item.emit(index);
                }}
                    >
                        { item.render() }
                    </a>
                </li>
            }
        })
        .collect::<Html>();
    html! {
        <div class="flex flex-wrap w-full justify-start border-2 border-solid">
            <div class="dropdown dropdown-bottom w-full">
                <input
                    ref={node.clone()}
                    tabindex=0
                    class={classes!("input", "w-full", "input-ghost")}
                    type="text"
                    value={view_ctx.value.clone()}
                    {oninput}
                    {onkeydown}
                    onkeydown={view_ctx.callbacks.on_keydown.clone()}
                />
                if !view_ctx.items.is_empty() {
                    <ul
                        tabindex="0"
                        class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-52"
                    >
                        { options }
                    </ul>
                }
            </div>
        </div>
    }
}
