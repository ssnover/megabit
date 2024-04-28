use crate::utils::Button;
use crate::websocket_provider::use_websocket;
use megabit_runner_msgs::ConsoleMessage;
use yew::{function_component, html, Callback, Html, Properties};

#[function_component(NextAppButton)]
pub fn next_app_button(_props: &NextAppButtonProperties) -> Html {
    let ws = use_websocket();

    let on_click = {
        let ws = ws.clone();
        Callback::from(move |_| {
            let msg = ConsoleMessage::NextApp;
            let msg = serde_json::to_vec(&msg).unwrap();
            ws.send_message(msg);
        })
    };

    html! {
        <Button text={"Next App >>"} on_click_cb={on_click} />
    }
}

#[derive(Properties, PartialEq)]
pub struct NextAppButtonProperties {}
