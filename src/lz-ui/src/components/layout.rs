use yew::prelude::*;

use super::CreateForm;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ModalState {
    /// No modals are visible
    Normal,

    /// "Create bookmark" modal visible
    CreateBookmark,
}

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Html, // the field name `children` is important!
}

#[function_component(Layout)]
pub fn layout(LayoutProps { children }: &LayoutProps) -> Html {
    let ctx = use_state(|| ModalState::Normal);
    html! {
        <ContextProvider<ModalState> context={*ctx}>
            <CreateForm onclose={{let ctx = ctx.clone(); move |_ev| ctx.set(ModalState::Normal)}} />
            <div class={classes!("navbar", "bg-base-100")}>
                <div class="navbar-start" />
                <div class="navbar-end">
                    <button class="btn" onclick={{move |_ev| ctx.set(ModalState::CreateBookmark)}}>
                        { "add" }
                    </button>
                </div>
            </div>
            { children.clone() }
        </ContextProvider<ModalState>>
    }
}
