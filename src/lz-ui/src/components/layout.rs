use bounce::prelude::*;
use patternfly_yew::prelude::*;
use yew::prelude::*;

use crate::dispatch_callback;

use super::CreateForm;

pub struct CloseModal;

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Slice)]
#[bounce(with_notion(CloseModal))]
pub enum ModalState {
    /// No modals are visible
    #[default]
    Normal,

    /// "Create bookmark" modal visible
    CreateBookmark,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ModalAction {
    OpenCreateBookmark,
    Close,
}

impl Reducible for ModalState {
    type Action = ModalAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match (action, &*self) {
            (ModalAction::OpenCreateBookmark, ModalState::Normal) => {
                ModalState::CreateBookmark.into()
            }
            (ModalAction::Close, _) => ModalState::Normal.into(),
            (ModalAction::OpenCreateBookmark, _) => self.into(),
        }
    }
}

impl WithNotion<CloseModal> for ModalState {
    fn apply(self: std::rc::Rc<Self>, _notion: std::rc::Rc<CloseModal>) -> std::rc::Rc<Self> {
        Default::default()
    }
}

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Html, // the field name `children` is important!
}

#[function_component(Layout)]
pub fn layout(LayoutProps { children }: &LayoutProps) -> Html {
    let state = use_slice::<ModalState>();
    let open_create = dispatch_callback(&state, |_| ModalAction::OpenCreateBookmark);
    let onclose = use_notion_applier::<CloseModal>();
    html! {
        <>
            <CreateForm onclose={move |_| onclose(CloseModal)} />
            <Grid gutter=true>
                <GridItem cols={[6]}>{ children.clone() }</GridItem>
                <GridItem cols={[2]}>
                    <Button label="Add" onclick={open_create} />
                </GridItem>
            </Grid>
        </>
    }
}
