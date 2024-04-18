use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_autocomplete::view::{Context, RenderHtml};

#[function_component(Tailwind)]
pub fn tailwind() -> Html {
    let view_ctx = use_context::<Context<String>>().expect("view::Context wasn't provided");

    let selected = view_ctx
        .selected_items
        .iter()
        .map(|value| {
            html! {
                <span class={classes!("badge", "badge-neutral")}>
                    <button
                        onclick={{
                            let value = value.clone();
                            move |_ev| {
                                tracing::info!(?value, ?index, "TODO: removing items is unimplemented");
                            }
                        }}
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            class="inline-block w-4 h-4 stroke-current"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M6 18L18 6M6 6l12 12"
                            />
                        </svg>
                    </button>
                    { value.render() }
                </span>
            }
        })
        .collect::<Html>();

    let input_cb = view_ctx.callbacks.on_input.clone();
    let oninput = move |e: InputEvent| {
        let input = e.target_dyn_into::<HtmlInputElement>().unwrap();
        input_cb.emit(input.value());
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
            if !view_ctx.selected_items.is_empty() {
                <span>{ selected }</span>
            }
            <div class="dropdown dropdown-bottom w-full">
                <input
                    tabindex=0
                    class={classes!("input", "w-full", "input-ghost")}
                    type="text"
                    value={view_ctx.value.clone()}
                    {oninput}
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
