use yew::{function_component, html, Callback, Html, MouseEvent, Properties};

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let props = props.clone();

    html! {
        <div style="margin: 10px">
            <button onclick={props.on_click_cb}>
                <p>{props.text}</p>
            </button>
        </div>
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ButtonProps {
    pub text: String,
    pub on_click_cb: Callback<MouseEvent>,
}
