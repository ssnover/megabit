use gloo::history::Location;
use yew::{function_component, html, Html};
use yew_router::{components::Link, hooks::use_location, Routable};

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Control,
    #[at("/installed")]
    Installed,
}

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let location = use_location();
    let control_class = get_nav_class("/", &location);
    let installed_class = get_nav_class("/installed", &location);

    html! {
        <div class="container bg-dark">
            <nav class="navbar navbar-expand-lg navbar-primary fixed-top border-dark">
                <Link<Route> classes={"navbar-brand text-primary"} to={Route::Control}>{ "megabit" }</Link<Route>>
                <button class="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
                    <span class="navbar-toggler-icon"></span>
                </button>

                <div class="collapse navbar-collapse" id="navbarSupportedContent">
                    <ul class="navbar-nav mr-auto">
                        <li class={ control_class }>
                            <Link<Route> classes={"nav-link text-primary"} to={Route::Control}>{ "Control" }</Link<Route>>
                        </li>
                        <li class={ installed_class }>
                            <Link<Route> classes={"nav-link text-primary"} to={Route::Installed}>{"Installed Apps"}</Link<Route>>
                        </li>
                    </ul>
                </div>
            </nav>
        </div>
    }
}

fn get_nav_class(prefix: &str, location: &Option<Location>) -> String {
    format!(
        "nav-item {}",
        location
            .as_ref()
            .map(|location| {
                if location.path() == prefix {
                    "active"
                } else {
                    ""
                }
            })
            .unwrap_or("")
    )
}
