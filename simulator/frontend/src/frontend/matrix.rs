use gloo::{events::EventListener, utils::window};
use js_sys::JsString;
use std::ops::Deref;
use wasm_bindgen::JsCast;
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
            style="background-color: #000000; padding: 10px;"
        >
        </canvas>
    }
}

#[derive(Properties, PartialEq)]
pub struct CanvasProperties {
    pub renderer: Callback<HtmlCanvasElement>,
    pub last_render_time: UseStateHandle<JsString>,
}

pub struct MatrixBuffer {
    data: Vec<[u8; 4]>,
    width: usize,
    height: usize,
}

impl MatrixBuffer {
    pub fn new(width: u32, height: u32) -> MatrixBuffer {
        MatrixBuffer {
            data: vec![[0x00, 0x00, 0x00, 0xff]; (width * height) as usize],
            width: width as usize,
            height: height as usize,
        }
    }

    pub fn to_megapixels(&self, pixels_per_cell: usize) -> Vec<u8> {
        let mut megapixels = vec![[0u8; 4]; pixels_per_cell * pixels_per_cell * self.data.len()];
        for row in 0..self.height as usize {
            for cell_row in 0..pixels_per_cell {
                for col in 0..self.width as usize {
                    for cell_col in 0..pixels_per_cell {
                        megapixels[cell_col
                            + (col * pixels_per_cell)
                            + (cell_row * self.width * pixels_per_cell)
                            + (row * pixels_per_cell * self.width * pixels_per_cell)] =
                            self.data[col + (self.width * row)];
                    }
                }
            }
        }

        megapixels.into_iter().flatten().collect()
    }
}

pub mod simple_display {
    use crate::frontend::{gauss_filter::create_alpha_filter, matrix::apply_alpha_filter};

    use super::{get_2d_canvas, MatrixBuffer};
    use core::cell::RefCell;
    use lazy_static::lazy_static;
    use web_sys::{HtmlCanvasElement, ImageData};

    pub const PIXELS_PER_CELL: u32 = 24;
    pub const COLUMNS: u32 = 32;
    pub const ROWS: u32 = 16;

    lazy_static! {
        static ref ALPHA_FILTER: Vec<u8> = create_alpha_filter(PIXELS_PER_CELL as usize);
    }

    pub fn draw(canvas: HtmlCanvasElement, matrix_buffer: &RefCell<MatrixBuffer>) {
        let interface = get_2d_canvas(&canvas);
        interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        let matrix_buffer = matrix_buffer.borrow();
        let mut image_data = matrix_buffer.to_megapixels(PIXELS_PER_CELL as usize);
        apply_alpha_filter(&mut image_data, PIXELS_PER_CELL, COLUMNS, &ALPHA_FILTER);
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&image_data),
            COLUMNS * PIXELS_PER_CELL,
            ROWS * PIXELS_PER_CELL,
        )
        .unwrap();
        if let Err(err) = interface.put_image_data(&image_data, 0.0, 0.0) {
            log::error!("Failed to render: {err:?}");
        }

        log::info!("Draw simple display");
    }

    pub fn update_row(row_number: u8, data: Vec<bool>, matrix_buffer: &RefCell<MatrixBuffer>) {
        let mut matrix_buffer = matrix_buffer.borrow_mut();
        let start_offset = row_number as usize * COLUMNS as usize;
        matrix_buffer.data[start_offset..(start_offset + data.len())]
            .iter_mut()
            .zip(data)
            .for_each(|(elem, new_state)| {
                if new_state {
                    *elem = [0xff, 0x00, 0x00, 0xff];
                } else {
                    *elem = [0x00, 0x00, 0x00, 0xff];
                }
            });
    }
}

pub mod rgb_display {
    use super::{apply_alpha_filter, get_2d_canvas, MatrixBuffer};
    use crate::frontend::gauss_filter::create_alpha_filter;
    use core::cell::RefCell;
    use lazy_static::lazy_static;
    use web_sys::{HtmlCanvasElement, ImageData};

    const PIXELS_PER_CELL: u32 = 12;
    pub const COLUMNS: u32 = 64;
    pub const ROWS: u32 = 32;

    lazy_static! {
        static ref ALPHA_FILTER: Vec<u8> = create_alpha_filter(PIXELS_PER_CELL as usize);
    }

    pub fn draw(canvas: HtmlCanvasElement, matrix_buffer: &RefCell<MatrixBuffer>) {
        static mut COUNTER: u64 = 0u64;
        let interface = get_2d_canvas(&canvas);

        let matrix_buffer = matrix_buffer.borrow();
        let mut image_data = matrix_buffer.to_megapixels(PIXELS_PER_CELL as usize);
        apply_alpha_filter(&mut image_data, PIXELS_PER_CELL, COLUMNS, &ALPHA_FILTER);
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&image_data),
            COLUMNS * PIXELS_PER_CELL,
            ROWS * PIXELS_PER_CELL,
        )
        .unwrap();
        if let Err(err) = interface.put_image_data(&image_data, 0.0, 0.0) {
            log::error!("Failed to render: {err:?}");
        }
        unsafe {
            log::info!("Draw: {COUNTER}");
            COUNTER = COUNTER.wrapping_add(1);
        }
    }

    pub fn update_row(row_number: u8, data: Vec<u16>, matrix_buffer: &RefCell<MatrixBuffer>) {
        let start_offset = row_number as usize * COLUMNS as usize;
        let mut matrix_buffer = matrix_buffer.borrow_mut();
        matrix_buffer.data[start_offset..(start_offset + data.len())]
            .iter_mut()
            .zip(data)
            .for_each(|(elem, new_color)| {
                let r = ((new_color & 0b11111_00000_00000) >> 10) as u8;
                let g = ((new_color & 0b00000_11111_00000) >> 5) as u8;
                let b = ((new_color & 0b00000_00000_11111) >> 0) as u8;
                *elem = [r << 3, g << 3, b << 3, 0xff];
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

fn apply_alpha_filter(image_data: &mut [u8], cell_size: u32, width: u32, alpha_filter: &[u8]) {
    image_data
        .iter_mut()
        .enumerate()
        .skip(3)
        .step_by(4)
        .for_each(|(idx, alpha_pixel)| {
            let row = (idx / 4) / (width * cell_size) as usize;
            let col = (idx / 4) % (width * cell_size) as usize;
            let cell_row = row % cell_size as usize;
            let cell_col = col % cell_size as usize;
            let filter_idx = (cell_row * cell_size as usize) + cell_col;
            *alpha_pixel = alpha_filter[filter_idx];
        })
}
