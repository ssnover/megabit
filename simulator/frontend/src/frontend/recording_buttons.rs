use crate::frontend::websocket_provider::use_websocket;
use megabit_sim_msgs::SimMessage;
use yew::prelude::*;

#[function_component(StartRecording)]
pub fn start_recording(_props: &StartRecordingProperties) -> Html {
    let ws = use_websocket();
    let node_ref = NodeRef::default();

    let on_press = {
        let ws = ws.clone();
        Callback::from(move |_| {
            ws.send_message(rmp_serde::to_vec(&SimMessage::StartRecording).unwrap())
        })
    };

    html! {
        <div style="margin: 10px">
            <button
                ref={node_ref}
                onclick={on_press}
            >
                <p>{"Start Recording"}</p>
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StartRecordingProperties {}

#[function_component(StopRecording)]
pub fn stop_recording(_props: &StopRecordingProperties) -> Html {
    let ws = use_websocket();
    let node_ref = NodeRef::default();

    let on_press = {
        let ws = ws.clone();
        Callback::from(move |_| {
            ws.send_message(rmp_serde::to_vec(&SimMessage::StopRecording).unwrap())
        })
    };

    html! {
        <div style="margin: 10px">
            <button
                ref={node_ref}
                onclick={on_press}
            >
                <p>{"Stop Recording"}</p>
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StopRecordingProperties {}
