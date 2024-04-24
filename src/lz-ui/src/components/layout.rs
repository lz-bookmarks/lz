use patternfly_yew::prelude::*;
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
            <Grid gutter=true>
                <GridItem cols={[6]}>{ children.clone() }</GridItem>
                <GridItem cols={[2]}>
                    <Button label="Add" onclick={move |_ev| ctx.set(ModalState::CreateBookmark)} />
                </GridItem>
            </Grid>
        </ContextProvider<ModalState>>
    }
}
