use yew::prelude::*;

use super::{ModalState, TagSelect};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onclose: Callback<MouseEvent>,
}

#[function_component(CreateForm)]
pub fn create_form(Props { onclose }: &Props) -> Html {
    let state = use_context::<ModalState>().expect("context needs to be provided");
    let tags = use_state_eq(|| vec![]);
    let onclick = {
        let tags = tags.clone();
        let onclose = onclose.clone();
        move |ev| {
            tags.set(vec![]);
            onclose.emit(ev);
        }
    };
    html! {
        if state == ModalState::CreateBookmark {
            <>
                <input
                    type="checkbox"
                    id="create_modal_visibility"
                    class="modal-toggle"
                    checked=true
                />
                <div class="modal" role="dialog">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{ "Add bookmark" }</h3>
                        <p class="py-4">{ "This modal works with a hidden checkbox!" }</p>
                        <TagSelect on_change={move |new_tags| {tags.set(new_tags)}} />
                        <div class="modal-action">
                            <button
                                {onclick}
                                class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                            >
                                { "✕" }
                            </button>
                        </div>
                    </div>
                </div>
            </>
        } else {
            <></>
        }
    }
}
