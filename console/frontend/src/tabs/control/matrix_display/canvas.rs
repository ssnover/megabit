use js_sys::JsString;
use std::cell::RefCell;
use web_sys::HtmlCanvasElement;
use yew::{
    function_component, html, use_effect, use_state, Callback, Html, NodeRef, Properties,
    UseStateHandle,
};

use super::buffer::MatrixBuffer;

#[function_component(Canvas)]
pub fn canvas(props: &CanvasProps) -> Html {
    let node_ref = NodeRef::default();
    let is_first_render = use_state(|| true);

    {
        let renderer = props.renderer.clone();
        let node_ref = node_ref.clone();
        use_effect(move || {
            if let Some(canvas) = node_ref.cast::<HtmlCanvasElement>() {
                if *is_first_render {
                    is_first_render.set(false);
                }

                renderer.emit(canvas);
            }

            || ()
        })
    }

    html! {
        <>
            <canvas
                ref={node_ref}
                width={ props.width.to_string() }
                height={ props.height.to_string() }
                style="background-color: #000000; padding: 10px; border-radius: 10px"
            />
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct CanvasProps {
    pub renderer: Callback<HtmlCanvasElement>,
    pub buffer: UseStateHandle<RefCell<MatrixBuffer>>,
    pub last_render_time: UseStateHandle<JsString>,
    pub width: u32,
    pub height: u32,
}
