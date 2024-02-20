use yew::prelude::*;

#[function_component(DebugLed)]
pub fn debug_led(props: &DebugLedProperties) -> Html {
    let node_ref = NodeRef::default();
    let (bg_color, text_color) = if *props.led_state {
        ("#00ff00", "#000000")
    } else {
        ("#000000", "#ffffff")
    };

    html! {
        <div
            style={ format!("background-color:{bg_color}; margin: 10px; border-radius: 15px") }
            width="20"
            height="20"
            ref={node_ref}
        >
            <p style={ format!("color:{text_color}") }>{ "DEBUG LED" }</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DebugLedProperties {
    pub led_state: UseStateHandle<bool>,
}
