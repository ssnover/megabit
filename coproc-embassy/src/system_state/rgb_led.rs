pub trait RgbLed {
    fn set_state(&mut self, r: u8, g: u8, b: u8);

    fn off(&mut self);
}
