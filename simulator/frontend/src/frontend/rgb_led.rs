use yew::prelude::*;

#[function_component(RgbLed)]
pub fn rgb_led(props: &RgbLedProperties) -> Html {
    let node_ref = NodeRef::default();
    let bg_color = hex_code_from_rgb(*props.rgb_state);
    let text_color = hex_code_from_rgb((
        255 - props.rgb_state.0,
        255 - props.rgb_state.1,
        255 - props.rgb_state.2,
    ));

    html! {
        <div
            style={ format!("background-color: {bg_color}; margin: 10px; border-radius: 15px") }
            ref={node_ref}
        >
            <p style={ format!("color: {text_color}")}>{ "STATUS LED" }</p>
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
