use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

pub mod simple {
    pub const SCREEN_WIDTH: usize = 32;
    pub const SCREEN_HEIGHT: usize = 16;
    pub type DisplayBuffer = super::DisplayBuffer<SCREEN_WIDTH, SCREEN_HEIGHT>;
}

mod imports {
    use extism_pdk::*;

    #[host_fn]
    extern "ExtismHost" {
        pub fn write_region(
            position_x: u32,
            position_y: u32,
            width: u32,
            height: u32,
            input_data: Vec<u8>,
        ) -> ();

        pub fn render(rows_to_update: Vec<u8>) -> ();
    }
}

pub fn write_region(
    position: (u32, u32),
    dimensions: (u32, u32),
    input_data: Vec<u8>,
) -> Result<(), extism_pdk::Error> {
    unsafe {
        imports::write_region(
            position.0,
            position.1,
            dimensions.0,
            dimensions.1,
            input_data,
        )
    }
}

pub fn render(rows_to_update: Vec<u8>) -> Result<(), extism_pdk::Error> {
    unsafe { imports::render(rows_to_update) }
}

pub struct DisplayBuffer<const WIDTH: usize, const HEIGHT: usize> {
    data: [[bool; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> DisplayBuffer<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            data: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        const BITS_PER_BYTE: usize = 8;

        let mut output = vec![0u8; WIDTH * HEIGHT / BITS_PER_BYTE];
        if (WIDTH * HEIGHT) % BITS_PER_BYTE != 0 {
            output.push(0);
        }

        for (row, row_data) in self.data.into_iter().enumerate() {
            for (col, elem) in row_data.into_iter().enumerate() {
                let idx = col + (row * WIDTH);
                if elem {
                    output[idx / BITS_PER_BYTE] |= 1 << (idx % BITS_PER_BYTE);
                }
            }
        }

        output
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> DrawTarget for DisplayBuffer<WIDTH, HEIGHT> {
    type Color = BinaryColor;
    type Error = ();

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        for row in &mut self.data {
            for col in row.into_iter() {
                *col = false;
            }
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x < 0 || coord.y < 0 {
                continue;
            }

            let (x, y) = (coord.x as usize, coord.y as usize);
            self.data[y][x] = color.is_on();
        }

        Ok(())
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> OriginDimensions for DisplayBuffer<WIDTH, HEIGHT> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
