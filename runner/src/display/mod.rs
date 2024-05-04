pub use megabit_serial_protocol::PixelRepresentation;
use megabit_utils::rgb555::Rgb555;
use std::{
    io,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    pub width: usize,
    pub height: usize,
    pub is_rgb: bool,
}

pub const DEFAULT_MONO_PALETTE: MonocolorPalette = MonocolorPalette::new(
    Rgb555::from_rgb(0xff, 0x00, 0x00),
    Rgb555::from_rgb(0x00, 0x00, 0x00),
);

#[derive(Debug, Clone)]
pub struct ScreenBufferHandle {
    inner: Arc<Mutex<ScreenBuffer>>,
}

#[derive(Debug, Clone, Copy)]
pub struct MonocolorPalette {
    on: Rgb555,
    off: Rgb555,
}

impl MonocolorPalette {
    pub const fn new(on: Rgb555, off: Rgb555) -> Self {
        Self { on, off }
    }

    pub fn from_on_color(color: Rgb555) -> Self {
        Self {
            on: color,
            off: 0x0000.into(),
        }
    }

    pub fn get_color(&self, value: bool) -> Rgb555 {
        if value {
            self.on
        } else {
            self.off
        }
    }
}

impl ScreenBufferHandle {
    pub fn is_rgb(&self) -> bool {
        let buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.is_rgb()
    }

    pub fn display_config(&self) -> DisplayConfiguration {
        let buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.display_config()
    }

    pub fn set_palette(&self, palette: MonocolorPalette) -> io::Result<()> {
        let mut buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.set_palette(palette)
    }

    pub fn set_cell(&self, row: usize, col: usize, value: bool) -> io::Result<()> {
        let mut buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.set_cell(row, col, value)
    }

    pub fn set_cell_rgb(&self, row: usize, col: usize, value: Rgb555) -> io::Result<()> {
        let mut buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.set_cell_rgb(row, col, value)
    }

    pub fn get_row(&self, row: usize) -> io::Result<(Vec<bool>, bool)> {
        let buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.get_row(row)
    }

    pub fn get_row_rgb(&self, row: usize) -> io::Result<(Vec<Rgb555>, bool)> {
        let buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.get_row_rgb(row)
    }

    pub fn clear_dirty_status(&self) {
        let mut buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.clear_dirty_status()
    }

    pub fn all_dirty(&self) {
        let mut buffer = self.inner.lock().expect("Mutex not poisoned");
        buffer.all_dirty()
    }
}

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    width: usize,
    height: usize,
    data: Vec<Rgb555>,
    palette: MonocolorPalette,
    dirty_row_buffer: Vec<bool>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![Rgb555::from_rgb(0x00, 0x00, 0x00); width * height],
            palette: MonocolorPalette::from_on_color(Rgb555::from_rgb(0xff, 0x00, 0x00)),
            dirty_row_buffer: vec![false; height],
        }
    }

    pub fn create(width: usize, height: usize) -> ScreenBufferHandle {
        let buffer = Self::new(width, height);
        ScreenBufferHandle {
            inner: Arc::new(Mutex::new(buffer)),
        }
    }

    pub fn is_rgb(&self) -> bool {
        true
    }

    pub fn display_config(&self) -> DisplayConfiguration {
        DisplayConfiguration {
            width: self.width,
            height: self.height,
            is_rgb: self.is_rgb(),
        }
    }

    pub fn set_palette(&mut self, new_palette: MonocolorPalette) -> io::Result<()> {
        self.palette = new_palette;
        self.all_dirty();
        Ok(())
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: bool) -> io::Result<()> {
        if row < self.height || col < self.width {
            let cell_idx = row * self.width + col;
            let old_color = self.data[cell_idx];
            let new_color = self.palette.get_color(value);
            if old_color != new_color {
                self.dirty_row_buffer[row] = true;
                self.data[cell_idx] = new_color;
            }
            Ok(())
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }

    pub fn set_cell_rgb(&mut self, row: usize, col: usize, value: Rgb555) -> io::Result<()> {
        if row < self.height || col < self.width {
            let cell_idx = row * self.width + col;
            if self.data[cell_idx] != value {
                self.dirty_row_buffer[row] = true;
                self.data[cell_idx] = value;
            }
            Ok(())
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }

    pub fn get_row(&self, row: usize) -> io::Result<(Vec<bool>, bool)> {
        if row < self.height {
            let start_idx = row * self.width;
            let row_data = self.data[start_idx..(start_idx + self.width)]
                .into_iter()
                .map(|color| *color == self.palette.on)
                .collect();
            Ok((row_data, self.dirty_row_buffer[row]))
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }

    pub fn get_row_rgb(&self, row: usize) -> io::Result<(Vec<Rgb555>, bool)> {
        if row < self.height {
            let start_idx = row * self.width;
            let row_data = self.data[start_idx..(start_idx + self.width)]
                .into_iter()
                .copied()
                .collect();
            Ok((row_data, self.dirty_row_buffer[row]))
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }

    pub fn clear_dirty_status(&mut self) {
        self.dirty_row_buffer
            .iter_mut()
            .for_each(|row| *row = false);
    }

    pub fn all_dirty(&mut self) {
        self.dirty_row_buffer.iter_mut().for_each(|row| *row = true);
    }
}
