use yew::prelude::*;

#[function_component(RgbLed)]
pub fn rgb_led(props: &RgbLedProperties) -> Html {
    let node_ref = NodeRef::default();

    html! {
        <div
            style={ format!("background-color: {}", hex_code_from_rgb(*props.rgb_state)) }
            ref={node_ref}
        >
            <p>{ "STATUS LED" }</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct RgbLedProperties {
    pub rgb_state: UseStateHandle<(u8, u8, u8)>,
}

fn hex_code_from_rgb((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}
