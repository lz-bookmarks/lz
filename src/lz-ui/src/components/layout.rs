use yew::prelude::*;
use yew_router::prelude::*;

use super::CreateForm;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ModalState {
    /// No modals are visible
    Normal,

    /// "Create bookmark" modal visible
    CreateBookmark,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Html, // the field name `children` is important!
}

#[function_component(Layout)]
pub fn layout(Props { children }: &Props) -> Html {
    let ctx = use_state(|| ModalState::Normal);
    html! {
        <ContextProvider<ModalState> context={(*ctx).clone()}>
            <CreateForm onclose={{let ctx = ctx.clone(); move |_ev| ctx.set(ModalState::Normal)}} />
            <button class="btn" onclick={{move |_ev| ctx.set(ModalState::CreateBookmark)}}>
                { "add" }
            </button>
            { children.clone() }
        </ContextProvider<ModalState>>
    }
}
