use megabit_serial_protocol::PixelRepresentation;

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
