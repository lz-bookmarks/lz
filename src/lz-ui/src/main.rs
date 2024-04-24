use bounce::BounceRoot;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use yew::prelude::*;
use yew_router::prelude::*;

use lz_ui::route::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </BounceRoot>
    }
}

fn main() {
    lz_ui::js::setup_sentry();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers, see note below
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events

    yew::Renderer::<App>::new().render();
}
