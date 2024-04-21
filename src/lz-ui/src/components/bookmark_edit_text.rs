use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, Default, PartialEq, Clone, Debug)]
pub struct BookmarkEditTextProps {
    pub name: String,
    pub id: String,
    pub multiline: bool,
    pub value: String,
    pub onchange: Callback<String>,
}

#[function_component(BookmarkEditText)]
pub fn bookmark_edit_text(
    BookmarkEditTextProps {
        name,
        id,
        multiline,
        value,
        onchange,
    }: &BookmarkEditTextProps,
) -> Html {
    html! {
        <div class="grid grid-cols-1 gap-1">
            <label class="font-medium" for={id.clone()}>{ name }</label>
            if !multiline {
                <input
                    id={id.clone()}
                    class={classes!("input", "input-bordered", "w-full", "max-w-xs")}
                    value={value.clone()}
                    oninput={let onchange = onchange.clone(); {move |ev: InputEvent| {
                        let input = ev.target_dyn_into::<HtmlInputElement>().unwrap();
                        onchange.clone().emit(input.value());
                    }}}
                />
            } else {
                <textarea
                    id={id.clone()}
                    class={classes!("input", "input-bordered", "w-full", "max-w-xs")}
                    value={value.clone()}
                    oninput={let onchange = onchange.clone(); {move |ev: InputEvent| {
                        let input = ev.target_dyn_into::<HtmlTextAreaElement>().unwrap();
                        onchange.emit(input.value());
                    }}}
                />
            }
        </div>
    }
}
