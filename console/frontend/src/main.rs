use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

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
    match routes {
        Route::Control => html! { <h1>{ "Control Panel" }</h1> },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::Renderer::<App>::new().render();
}
