use crate::host;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

pub fn write_region(
    position: (u32, u32),
    dimensions: (u32, u32),
    input_data: Vec<u8>,
) -> Result<(), extism_pdk::Error> {
    unsafe {
        host::write_region(
            position.0,
            position.1,
            dimensions.0,
            dimensions.1,
            input_data,
        )
    }
}

pub fn render(rows_to_update: Vec<u8>) -> Result<(), extism_pdk::Error> {
    unsafe { host::render(rows_to_update) }
}

pub fn set_monocolor_palette(on_color: u16, off_color: u16) -> Result<(), extism_pdk::Error> {
    unsafe { host::set_monocolor_palette(on_color.into(), off_color.into()) }
}

#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    pub width: usize,
    pub height: usize,
    pub is_rgb: bool,
}

pub fn get_display_info() -> Result<DisplayConfiguration, extism_pdk::Error> {
    let raw_info = unsafe { host::get_display_info() }?;
    let width = u32::from_be_bytes(raw_info[0..4].try_into()?);
    let height = u32::from_be_bytes(raw_info[4..8].try_into()?);
    let is_rgb = raw_info[8] != 0;
    Ok(DisplayConfiguration {
        width: width as usize,
        height: height as usize,
        is_rgb,
    })
}

pub type MonocolorBuffer = DisplayBuffer<bool>;
pub type RgbBuffer = DisplayBuffer<u16>;

pub struct DisplayBuffer<T: Copy + Default> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T: Copy + Default> DisplayBuffer<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![T::default(); width * height],
            width,
            height,
        }
    }
}

impl DrawTarget for MonocolorBuffer {
    type Color = BinaryColor;
    type Error = ();

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        for row in 0..self.height {
            for col in 0..self.width {
                self.data[row * self.width + col] = BinaryColor::Off.is_on();
            }
        }

        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            if (0..(self.height as isize)).contains(&(y as isize))
                && (0..(self.width as isize)).contains(&(x as isize))
            {
                let row = y as usize;
                let col = x as usize;
                self.data[row * self.width + col] = color.is_on();
            }
        }

        Ok(())
    }
}

impl OriginDimensions for MonocolorBuffer {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
