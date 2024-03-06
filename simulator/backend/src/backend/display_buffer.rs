const DEFAULT_MONOCOLOR: u16 = 0b11111_00000_00000;

pub struct DisplayBuffer {
    width: usize,
    height: usize,
    buffer: Vec<u16>,
    monocolor: u16,
}

impl DisplayBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        DisplayBuffer {
            width,
            height,
            buffer: vec![0; width * height],
            monocolor: DEFAULT_MONOCOLOR,
        }
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_frame(&self) -> &[u16] {
        &self.buffer
    }

    pub fn set_monocolor_palette(&mut self, monocolor: u16) {
        self.monocolor = monocolor;
    }

    pub fn get_monocolor(&self) -> u16 {
        self.monocolor
    }

    pub fn update_row(&mut self, row_number: u8, data: Vec<bool>) {
        let row_number = row_number as usize;
        let start_idx = row_number * self.width;
        for (idx, new_value) in (start_idx..(start_idx + self.width))
            .into_iter()
            .zip(data.iter())
        {
            self.buffer[idx] = if *new_value { self.monocolor } else { 0 };
        }
    }

    pub fn update_row_rgb(&mut self, row_number: u8, data: Vec<u16>) {
        let row_number = row_number as usize;
        let start_idx = row_number * self.width;
        for (idx, new_value) in (start_idx..(start_idx + self.width))
            .into_iter()
            .zip(data.iter())
        {
            self.buffer[idx] = *new_value;
        }
    }
}
