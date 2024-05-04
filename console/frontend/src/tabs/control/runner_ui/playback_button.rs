use crate::providers::use_websocket;
use crate::utils::Button;
use megabit_runner_msgs::ConsoleMessage;
use yew::{function_component, html, Callback, Html, Properties};

#[function_component(PlaybackButton)]
pub fn playback_button(_props: &PlaybackButtonProperties) -> Html {
    let ws = use_websocket();

    let on_click = {
        let ws = ws.clone();
        Callback::from(move |_| {
            let msg = ConsoleMessage::ResumeRendering;
            let msg = serde_json::to_vec(&msg).unwrap();
            ws.send_message(msg);
        })
    };

    html! {
        <Button text={"Play/Pause Button"} on_click_cb={on_click} />
    }
}

#[derive(Properties, PartialEq)]
pub struct PlaybackButtonProperties {}
