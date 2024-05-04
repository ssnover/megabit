use next_app_button::NextAppButton;
use playback_button::PlaybackButton;
use prev_app_button::PrevAppButton;
use yew::{function_component, html, Html};

mod next_app_button;
mod playback_button;
mod prev_app_button;

#[function_component(RunnerUi)]
pub fn runner_ui() -> Html {
    html! {
        <>
            <div class="col justify-content-center" style="display:grid">
                <PrevAppButton/>
            </div>
            <div class="col justify-content-center" style="display:grid">
                <PlaybackButton/>
            </div>
            <div class="col justify-content-center" style="display:grid">
                <NextAppButton/>
            </div>
        </>
    }
}
