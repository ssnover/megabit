use crate::providers::{use_app_config, use_subscription_manager};
use buffer::MatrixBuffer;
use canvas::Canvas;
use gloo::{events::EventListener, utils::window};
use js_sys::JsString;
use megabit_runner_msgs::ConsoleMessage;
use std::cell::RefCell;
use web_sys::HtmlDivElement;
use yew::{function_component, html, use_state, Callback, Html, NodeRef, Properties};

mod buffer;
mod canvas;

#[function_component(MatrixDisplay)]
pub fn matrix_display(props: &MatrixDisplayProps) -> Html {
    let app_cfg = use_app_config();
    let parent_size = {
        let node_ref = props.div_ref.clone();
        use_state(|| get_size(node_ref))
    };

    let _size_listener = {
        let parent_size = parent_size.clone();
        let node_ref = props.div_ref.clone();
        use_state(move || {
            EventListener::new(&window(), "resize", move |_| {
                let node_ref = node_ref.clone();
                log::debug!("Window resize event");
                parent_size.set(get_size(node_ref))
            })
        })
    };

    let pixels_per_matrix_cell = use_state(|| (*parent_size).0 / app_cfg.width());
    let buffer = use_state(|| {
        RefCell::new(MatrixBuffer::new(
            app_cfg.width(),
            app_cfg.height(),
            *pixels_per_matrix_cell as usize,
        ))
    });
    let last_render_time = use_state(|| get_time());

    let renderer_cb = {
        let buffer = buffer.clone();
        Callback::from(move |canvas| {
            let buffer = buffer.borrow();
            buffer.draw(canvas)
        })
    };

    let sub_manager = use_subscription_manager();
    let _subscriptions = use_state(|| {
        let buffer = buffer.clone();
        sub_manager.subscribe(
            "matrix_display",
            "SetMatrixRowRgb",
            Callback::from(move |msg| {
                if let ConsoleMessage::SetMatrixRowRgb(data) = msg {
                    let mut buffer = buffer.borrow_mut();
                    log::debug!("Updating row");
                    buffer.update_row(data.row as u8, data.data);
                }
            }),
        );
        let last_render_time = last_render_time.clone();
        sub_manager.subscribe(
            "matrix_display",
            "CommitRender",
            Callback::from(move |msg| {
                if let ConsoleMessage::CommitRender = msg {
                    log::debug!("Committing render!");
                    last_render_time.set(get_time());
                }
            }),
        );
        ()
    });

    let canvas_width = (parent_size.0 as f32 * 0.8) as u32;
    let canvas_height = canvas_width / 2;

    html! {
        <div class="justify-content-center" style="display:grid">
            <Canvas renderer={renderer_cb} width={canvas_width} height={canvas_height} {buffer} {last_render_time} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MatrixDisplayProps {
    pub div_ref: NodeRef,
}

fn get_size(node_ref: NodeRef) -> (u32, u32) {
    log::debug!("{node_ref:?}");
    if let Some(element) = node_ref.cast::<HtmlDivElement>() {
        let size = (
            element.client_width() as u32,
            element.client_height() as u32,
        );
        log::info!("Calculated size: {}px by {}px", size.0, size.1);
        size
    } else {
        (640, 320)
    }
}

fn get_time() -> JsString {
    js_sys::Date::new_0().to_utc_string()
}
