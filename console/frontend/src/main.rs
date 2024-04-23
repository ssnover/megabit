use websocket_provider::WebsocketProvider;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod tabs;
mod websocket_provider;

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
        Route::Control => html! { <tabs::ControlPage></tabs::ControlPage> },
    };

    html! {
        <WebsocketProvider>
            { page_contents }
        </WebsocketProvider>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::Renderer::<App>::new().render();
}
