use next_app_button::NextAppButton;
use playback_button::PlaybackButton;
use prev_app_button::PrevAppButton;
use yew::{function_component, html, Html};

mod next_app_button;
mod playback_button;
mod prev_app_button;

#[function_component(ControlPage)]
pub fn control_page() -> Html {
    html! {
        <div class="container">
            <div class="row row-cols-1 align-items-center">
                <h1>{ "Control Page" }</h1>
            </div>
            <div class="row row-cols-3 align-items-center">
                <div class="col justify-content-center" style="display:grid">
                    <PrevAppButton/>
                </div>
                <div class="col justify-content-center" style="display:grid">
                    <PlaybackButton/>
                </div>
                <div class="col justify-content-center" style="display:grid">
                    <NextAppButton/>
                </div>
            </div>
        </div>

    }
}
