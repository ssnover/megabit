use providers::ConsoleProviders;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod providers;
mod tabs;
mod utils;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Control,
}

fn switch(routes: Route) -> Html {
    let page_contents = match routes {
        Route::Control => html! { <tabs::ControlPage /> },
    };

    html! {
        <ConsoleProviders>
            { page_contents }
        </ConsoleProviders>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    yew::Renderer::<App>::new().render();
}
