use gloo::{events::EventListener, utils::window};
use std::ops::Deref;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[function_component(Canvas)]
pub fn canvas(props: &CanvasProperties) -> Html {
    let node_ref = NodeRef::default();
    let is_first_render = use_state(|| true);
    let display_size = use_state(|| {
        (
            simple_display::COLUMNS * simple_display::PIXELS_PER_CELL,
            simple_display::ROWS * simple_display::PIXELS_PER_CELL,
        )
    });

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
    pub matrix_buffer: UseStateHandle<core::cell::RefCell<simple_display::MatrixBuffer>>,
    pub rgb_matrix_buffer: UseStateHandle<core::cell::RefCell<rgb_display::MatrixBuffer>>,
}

pub mod simple_display {
    use core::cell::RefCell;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    pub const PIXELS_PER_CELL: u32 = 16;
    pub const COLUMNS: u32 = 32;
    pub const ROWS: u32 = 16;
    pub type MatrixBuffer = Vec<u8>;

    pub fn draw(canvas: HtmlCanvasElement, matrix_buffer: &RefCell<MatrixBuffer>) {
        let interface = get_2d_canvas(&canvas);
        interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        let on_color: JsValue = JsValue::from("#ff0000");
        let off_color: JsValue = JsValue::from("#000000");

        let matrix_buffer = matrix_buffer.borrow();
        for row in (0..ROWS).into_iter() {
            for col in (0..COLUMNS).into_iter() {
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
        let mut matrix_buffer = matrix_buffer.borrow_mut();
        let start_offset = row_number as usize * COLUMNS as usize;
        matrix_buffer[start_offset..(start_offset + data.len())]
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
}

pub mod rgb_display {
    use core::cell::RefCell;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    const PIXELS_PER_CELL: u32 = 8;
    pub const COLUMNS: u32 = 64;
    pub const ROWS: u32 = 32;
    pub type MatrixBuffer = Vec<u16>;

    pub fn draw(canvas: HtmlCanvasElement, matrix_buffer: &RefCell<MatrixBuffer>) {
        let interface = get_2d_canvas(&canvas);

        let matrix_buffer = matrix_buffer.borrow();
        for row in (0..ROWS).into_iter() {
            for col in (0..COLUMNS).into_iter() {
                let rgb555_color = matrix_buffer[(row * COLUMNS + col) as usize];
                let (r, g, b) = {
                    let r = ((rgb555_color & 0b11111_00000_00000) >> 10) as u8;
                    let g = ((rgb555_color & 0b00000_11111_00000) >> 5) as u8;
                    let b = (rgb555_color & 0b00000_00000_11111) as u8;
                    (r << 3, g << 3, b << 3)
                };
                let color = JsValue::from(format!("#{r:02x}{g:02x}{b:02x}"));

                interface.set_fill_style(&color);
                interface.fill_rect(
                    (col * PIXELS_PER_CELL) as f64,
                    (row * PIXELS_PER_CELL) as f64,
                    PIXELS_PER_CELL as f64,
                    PIXELS_PER_CELL as f64,
                );
            }
        }
    }

    pub fn update_row(row_number: u8, data: Vec<u16>, matrix_buffer: &RefCell<MatrixBuffer>) {
        let start_offset = row_number as usize * COLUMNS as usize;
        let mut matrix_buffer = matrix_buffer.borrow_mut();
        matrix_buffer[start_offset..(start_offset + data.len())]
            .iter_mut()
            .zip(data)
            .for_each(|(elem, new_color)| {
                *elem = new_color;
            });
    }
}

fn get_2d_canvas(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap()
}
