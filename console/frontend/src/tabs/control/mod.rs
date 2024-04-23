use playback_button::PlaybackButton;
use yew::{function_component, html, Html};

mod playback_button;

#[function_component(ControlPage)]
pub fn control_page() -> Html {
    html! {
        <div>
            <h1>{ "Control Page" }</h1>
            <PlaybackButton></PlaybackButton>
        </div>

    }
}
