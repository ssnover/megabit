use crate::frontend::websocket_provider::use_websocket;
use megabit_sim_msgs::SimMessage;
use yew::prelude::*;

#[function_component(UserButton)]
pub fn user_button(_props: &UserButtonProperties) -> Html {
    let ws = use_websocket();
    let node_ref = NodeRef::default();

    let on_press = {
        let ws = ws.clone();
        Callback::from(move |_| {
            ws.send_message(rmp_serde::to_vec(&SimMessage::ReportButtonPress).unwrap())
        })
    };

    html! {
        <div style="margin: 10px">
            <button
                ref={node_ref}
                onclick={on_press}
            >
                <p>{"User Button"}</p>
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct UserButtonProperties {}
