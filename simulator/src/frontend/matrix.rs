use gloo::{events::EventListener, utils::window};
use std::{cell::RefCell, ops::Deref};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

const PIXELS_PER_CELL: u32 = 16;
pub const COLUMNS: u32 = 32;
pub const ROWS: u32 = 16;
pub type MatrixBuffer = [u8; (COLUMNS * ROWS) as usize];

#[function_component(Canvas)]
pub fn canvas(props: &CanvasProperties) -> Html {
    let node_ref = NodeRef::default();
    let is_first_render = use_state(|| true);
    let display_size = use_state(|| (COLUMNS * PIXELS_PER_CELL, ROWS * PIXELS_PER_CELL));

    let size_listen_event_state = use_state(|| EventListener::new(&window(), "resize", |_| ()));

    let node_ref_clone = node_ref.clone();
    let display_size_handle = display_size.clone();
    let renderer = props.renderer.clone();

    use_effect(move || {
        if let Some(canvas) = node_ref_clone.cast::<HtmlCanvasElement>() {
            if *is_first_render {
                is_first_render.set(false);
                let canvas = canvas.clone();

                display_size_handle
                    .set((canvas.client_width() as u32, canvas.client_height() as u32));

                size_listen_event_state.set(EventListener::new(&window(), "resize", move |_| {
                    display_size_handle
                        .set((canvas.client_width() as u32, canvas.client_height() as u32));
                }))
            }

            renderer.emit(canvas);
        }

        || ()
    });

    html! {
        <canvas
            width={display_size.clone().deref().0.to_string()}
            height={display_size.deref().1.to_string()}
            ref={node_ref}
        >
        </canvas>
    }
}

#[derive(Properties, PartialEq)]
pub struct CanvasProperties {
    pub renderer: Callback<HtmlCanvasElement>,
    pub counter: UseStateHandle<u64>,
}

pub fn draw(canvas: HtmlCanvasElement, matrix_buffer: &RefCell<MatrixBuffer>) {
    let interface: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

    let on_color: JsValue = JsValue::from("#ff0000");
    let off_color: JsValue = JsValue::from("#000000");

    for row in (0..(canvas.height() / PIXELS_PER_CELL)).into_iter() {
        for col in (0..(canvas.width() / PIXELS_PER_CELL)).into_iter() {
            let matrix_buffer = matrix_buffer.borrow();
            if matrix_buffer[(row * COLUMNS + col) as usize] != 0x00 {
                interface.set_fill_style(&on_color);
            } else {
                interface.set_fill_style(&off_color);
            }
            interface.fill_rect(
                (col * PIXELS_PER_CELL) as f64,
                (row * PIXELS_PER_CELL) as f64,
                PIXELS_PER_CELL as f64,
                PIXELS_PER_CELL as f64,
            );
        }
    }
}

pub fn update_row(row_number: u8, data: Vec<bool>, matrix_buffer: &RefCell<MatrixBuffer>) {
    let mut buffer = matrix_buffer.borrow_mut();
    let start_offset = row_number as usize * COLUMNS as usize;
    buffer[start_offset..(start_offset + data.len())]
        .iter_mut()
        .zip(data)
        .for_each(|(elem, new_state)| {
            if new_state {
                *elem = 0x01;
            } else {
                *elem = 0x00;
            }
        });
}
