use yew::prelude::*;

#[function_component(DebugLed)]
pub fn debug_led(props: &DebugLedProperties) -> Html {
    let node_ref = NodeRef::default();

    html! {
        <div
            style={ if *props.led_state { "background-color: #00ff00" } else { "background-color:#000000" } }
            width="20"
            height="20"
            ref={node_ref}
        >
            <p>{ "DEBUG LED" }</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DebugLedProperties {
    pub led_state: UseStateHandle<bool>,
}
