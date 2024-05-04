#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb555(pub u16);

impl Rgb555 {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(
            ((r as u16 & 0xf8) << 7) // R: Shift right by 3, then left by 10
                | ((g as u16 & 0xf8) << 2) // G: Shift right by 3, then left by 5
                | ((b as u16 & 0xf8) >> 3), // B: Shift right by 3, then left by 0
        )
    }
}

impl From<u16> for Rgb555 {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Rgb555> for u16 {
    fn from(value: Rgb555) -> Self {
        value.0
    }
}

impl From<[u8; 3]> for Rgb555 {
    fn from(value: [u8; 3]) -> Self {
        Self::from_rgb(value[0], value[1], value[2])
    }
}

impl From<Rgb555> for [u8; 3] {
    fn from(value: Rgb555) -> Self {
        let r = ((value.0 & 0b11111_00000_00000) >> 10) as u8;
        let g = ((value.0 & 0b00000_11111_00000) >> 5) as u8;
        let b = ((value.0 & 0b00000_00000_11111) >> 0) as u8;
        [r << 3, g << 3, b << 3]
    }
}

impl From<[u8; 4]> for Rgb555 {
    fn from(value: [u8; 4]) -> Self {
        Self::from_rgb(value[0], value[1], value[2])
    }
}

impl From<Rgb555> for [u8; 4] {
    fn from(value: Rgb555) -> Self {
        let r = ((value.0 & 0b11111_00000_00000) >> 10) as u8;
        let g = ((value.0 & 0b00000_11111_00000) >> 5) as u8;
        let b = ((value.0 & 0b00000_00000_11111) >> 0) as u8;
        [r << 3, g << 3, b << 3, 0xff]
    }
}
