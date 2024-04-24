use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/package.js")]
extern "C" {
    /// Setup Sentry's wasm reporting with the DSN configured during build time.
    pub fn setup_sentry();
}

#[wasm_bindgen(module = "/package.js")]
extern "C" {
    /// Duck typed autocomplete return value, containing hooks that
    /// drive various component states.
    pub type Autocomplete;

    /// Returns the properties that the <input> field should set.
    #[wasm_bindgen(structural, method)]
    pub fn getInputProps(this: &Autocomplete) -> js_sys::Map;

    /// Constructor for an [`Autocomplete`].
    pub type AutocompleteBuilder;

    #[wasm_bindgen(constructor)]
    pub fn new(getSources: &Closure<dyn FnMut(String) -> JsValue>) -> AutocompleteBuilder;

    #[wasm_bindgen(method)]
    pub fn build(this: &AutocompleteBuilder) -> Autocomplete;
}
