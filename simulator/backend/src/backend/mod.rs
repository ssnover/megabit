use megabit_serial_protocol::PixelRepresentation;

mod display_buffer;
mod recorder;
pub mod serial;
pub mod simulator;
pub mod web_server;

#[derive(Clone, Copy, Debug)]
pub struct DisplayConfiguration {
    pub is_rgb: bool,
    pub width: u32,
    pub height: u32,
}

impl From<DisplayConfiguration> for megabit_serial_protocol::GetDisplayInfoResponse {
    fn from(value: DisplayConfiguration) -> Self {
        megabit_serial_protocol::GetDisplayInfoResponse {
            width: value.width,
            height: value.height,
            pixel_representation: if value.is_rgb {
                PixelRepresentation::RGB555
            } else {
                PixelRepresentation::Monocolor
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color(u16);

impl Color {
    pub fn to_rgb(self) -> [u8; 3] {
        let r = ((self.0 & 0b11111_00000_00000) >> 10) as u8;
        let g = ((self.0 & 0b00000_11111_00000) >> 5) as u8;
        let b = (self.0 & 0b00000_00000_11111) as u8;
        [r << 3, g << 3, b << 3]
    }
}
