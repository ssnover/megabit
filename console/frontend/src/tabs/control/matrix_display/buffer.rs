use megabit_utils::rgb555::Rgb555;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[derive(Clone, PartialEq)]
pub struct MatrixBuffer {
    data: Vec<[u8; 4]>,
    width: usize,
    height: usize,
    pixels_per_cell: usize,
    alpha_filter: Vec<u8>,
}

impl MatrixBuffer {
    pub fn new(width: u32, height: u32, pixels_per_cell: usize) -> Self {
        Self {
            data: vec![[0x00, 0x00, 0x00, 0xff]; (width * height) as usize],
            width: width as usize,
            height: height as usize,
            pixels_per_cell,
            alpha_filter: create_alpha_filter(pixels_per_cell),
        }
    }

    pub fn draw(&self, canvas: HtmlCanvasElement) {
        let interface = get_2d_canvas(&canvas);
        interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        let mut image_data = self.to_megapixels();
        apply_alpha_filter(
            &mut image_data,
            self.pixels_per_cell,
            self.width,
            &self.alpha_filter,
        );
        if let Ok(image_data) = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&image_data),
            (self.width * self.pixels_per_cell) as u32,
            (self.height * self.pixels_per_cell) as u32,
        ) {
            if let Err(err) = interface.put_image_data(&image_data, 0.0, 0.0) {
                log::error!("Failed to render: {err:?}");
            }
        }
    }

    pub fn update_row(&mut self, row_number: u8, data: Vec<u16>) {
        let start_offset = row_number as usize * self.width;
        self.data
            .iter_mut()
            .skip(start_offset)
            .zip(data)
            .for_each(|(elem, new_color)| {
                let new_color = Rgb555(new_color);
                *elem = new_color.into();
            })
    }

    fn to_megapixels(&self) -> Vec<u8> {
        let mut megapixels =
            vec![[0u8; 4]; self.pixels_per_cell * self.pixels_per_cell * self.data.len()];
        for row in 0..self.height as usize {
            for cell_row in 0..self.pixels_per_cell {
                for col in 0..self.width as usize {
                    for cell_col in 0..self.pixels_per_cell {
                        megapixels[cell_col
                            + (col * self.pixels_per_cell)
                            + (cell_row * self.width * self.pixels_per_cell)
                            + (row * self.pixels_per_cell * self.width * self.pixels_per_cell)] =
                            self.data[col + (self.width * row)];
                    }
                }
            }
        }

        megapixels.into_iter().flatten().collect()
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

fn apply_alpha_filter(image_data: &mut [u8], cell_size: usize, width: usize, alpha_filter: &[u8]) {
    image_data
        .iter_mut()
        .enumerate()
        .skip(3)
        .step_by(4)
        .for_each(|(idx, alpha_pixel)| {
            let row = (idx / 4) / (width * cell_size);
            let col = (idx / 4) % (width * cell_size);
            let cell_row = row % cell_size;
            let cell_col = col % cell_size;
            let filter_idx = (cell_row * cell_size) + cell_col;
            *alpha_pixel = alpha_filter[filter_idx];
        })
}

fn create_alpha_filter(size: usize) -> Vec<u8> {
    let radius = size / 2;
    let mut data = vec![0u8; size * size];
    let center = f64::from(size as u32) / 2.0;

    const SHIFT: f64 = 0.0;
    const SCALAR: f64 = 4.0;

    for row in 0..size {
        for col in 0..size {
            let pixel = &mut data[(row * size) + col];
            let sigma_squared = f64::from(radius as u32) / 1.5;
            *pixel = (255.0
                * ((sigma_squared
                    * SCALAR
                    * (calc_gauss(
                        sigma_squared.powf(2.0),
                        dist((col as f64, row as f64), (center, center)),
                    )))
                    + SHIFT)) as u8;
        }
    }

    data
}

fn calc_gauss(sigma_squared: f64, x: f64) -> f64 {
    use std::f64::consts::PI;

    let scale = 1.0 / ((sigma_squared * 2.0 * PI).powf(0.5));
    let exponent = (-1.0 * x.powi(2)) / (2.0 * sigma_squared);
    scale * exponent.exp()
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).powf(0.5)
}
