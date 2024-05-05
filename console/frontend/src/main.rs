use navbar::Navbar;
use providers::ConsoleProviders;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod navbar;
mod providers;
mod tabs;
mod utils;

#[function_component(App)]
fn app() -> Html {
    html! {
        <ConsoleProviders>
            <BrowserRouter>
                <Switch<navbar::Route> render={switch} />
            </BrowserRouter>
        </ConsoleProviders>
    }
}

fn switch(route: navbar::Route) -> Html {
    let page_contents = match route {
        navbar::Route::Control => html! { <tabs::ControlPage /> },
        navbar::Route::Installed => html! { <p>{ "Installed Apps" }</p> },
    };

    html! {
        <>
            <Navbar/>
            <div style="height: 100%; background-color: var(--bs-primary-text-emphasis) !important">
                { page_contents }
            </div>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    yew::Renderer::<App>::new().render();
}
