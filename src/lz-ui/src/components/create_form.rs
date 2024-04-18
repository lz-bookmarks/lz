use itertools::Itertools as _;
use yew::prelude::*;
use yew_autocomplete::{view::Bulma, Autocomplete, ItemResolver, ItemResolverResult};
use yew_router::prelude::*;

use super::{ModalState, Tailwind};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onclose: Callback<MouseEvent>,
}

#[function_component(CreateForm)]
pub fn create_form(Props { onclose }: &Props) -> Html {
    let state = use_context::<ModalState>().expect("context needs to be provided");
    let onchange = |selected: Vec<String>| (tracing::info!(?selected, "got elements"));

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
        <>
            <input
                type="checkbox"
                id="create_modal_visibility"
                class="modal-toggle"
                checked={state == ModalState::CreateBookmark}
            />
            <div class="modal" role="dialog">
                <div class="modal-box">
                    <h3 class="font-bold text-lg">{ "Add bookmark" }</h3>
                    <p class="py-4">{ "This modal works with a hidden checkbox!" }</p>
                    <Autocomplete<String>
                        {onchange}
                        {resolve_items}
                        multi_select=true
                        auto=true
                        show_selected=true
                    >
                        <Tailwind />
                    </Autocomplete<String>>
                    <div class="modal-action">
                        <button onclick={onclose} class="btn">{ "Cancel" }</button>
                    </div>
                </div>
            </div>
        </>
    }
}
