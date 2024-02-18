pub use megabit_serial_protocol::PixelRepresentation;
use std::io;

#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    pub width: usize,
    pub height: usize,
    pub is_rgb: bool,
}

pub const DEFAULT_MONO_PALETTE: MonocolorPalette =
    MonocolorPalette::new(0b11111_00000_00000, 0x0000);

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    buffer: ScreenBufferKind,
    width: usize,
    height: usize,
    dirty_row_buffer: Vec<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct MonocolorPalette {
    on: u16,
    off: u16,
}

impl MonocolorPalette {
    pub const fn new(on: u16, off: u16) -> Self {
        Self { on, off }
    }

    pub fn from_on_color(color: u16) -> Self {
        Self {
            on: color,
            off: 0x0000,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScreenBufferKind {
    Monocolor(Vec<bool>),
    Rgb555(Vec<u16>, MonocolorPalette),
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize, rgb_monocolor: Option<MonocolorPalette>) -> Self {
        ScreenBuffer {
            buffer: if let Some(palette) = rgb_monocolor {
                ScreenBufferKind::Rgb555(vec![0u16; width * height], palette)
            } else {
                ScreenBufferKind::Monocolor(vec![false; width * height])
            },
            width,
            height,
            dirty_row_buffer: vec![false; height],
        }
    }

    pub fn is_rgb(&self) -> bool {
        matches!(self.buffer, ScreenBufferKind::Rgb555(_, _))
    }

    pub fn display_config(&self) -> DisplayConfiguration {
        DisplayConfiguration {
            width: self.width,
            height: self.height,
            is_rgb: self.is_rgb(),
        }
    }

    pub fn set_palette(&mut self, palette: MonocolorPalette) -> io::Result<()> {
        if let ScreenBufferKind::Rgb555(_, current_palette) = &mut self.buffer {
            *current_palette = palette;
            Ok(())
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: bool) -> io::Result<()> {
        if row >= self.height || col >= self.width {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        let index = row * self.width + col;
        match &mut self.buffer {
            ScreenBufferKind::Monocolor(ref mut buffer) => {
                if buffer[index] != value {
                    self.dirty_row_buffer[row] = true;
                }
                buffer[index] = value;
            }
            ScreenBufferKind::Rgb555(ref mut buffer, palette) => {
                let value = if value { palette.on } else { palette.off };
                if buffer[index] != value {
                    self.dirty_row_buffer[row] = true;
                }
                buffer[index] = value;
            }
        }

        Ok(())
    }

    pub fn set_cell_rgb(&mut self, row: usize, col: usize, value: u16) -> io::Result<()> {
        if row >= self.height || col >= self.width {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        let index = row * self.width + col;
        match &mut self.buffer {
            ScreenBufferKind::Rgb555(ref mut buffer, _) => {
                if buffer[index] != value {
                    self.dirty_row_buffer[row] = true;
                }
                buffer[index] = value;
                Ok(())
            }
            ScreenBufferKind::Monocolor(_) => Err(io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn get_row(&self, row_number: usize) -> io::Result<(Vec<bool>, bool)> {
        if row_number >= self.height {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        match &self.buffer {
            ScreenBufferKind::Monocolor(buffer) => {
                let start_idx = row_number * self.width;
                let end_idx = (row_number + 1) * self.width;
                Ok((
                    Vec::from(&buffer[start_idx..end_idx]),
                    self.dirty_row_buffer[row_number],
                ))
            }
            ScreenBufferKind::Rgb555(_, _) => Err(io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn get_row_rgb(&self, row_number: usize) -> io::Result<(Vec<u16>, bool)> {
        if row_number >= self.height {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        match &self.buffer {
            ScreenBufferKind::Rgb555(buffer, _) => {
                let start_idx = row_number * self.width;
                let end_idx = (row_number + 1) * self.width;
                Ok((
                    Vec::from(&buffer[start_idx..end_idx]),
                    self.dirty_row_buffer[row_number],
                ))
            }
            ScreenBufferKind::Monocolor(_) => Err(io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn clear_dirty_status(&mut self) {
        self.dirty_row_buffer
            .iter_mut()
            .for_each(|row| *row = false);
    }
}
