use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};
use extism_pdk::*;
use megabit_app_sdk::display::{
    self,
    simple::{DisplayBuffer, SCREEN_HEIGHT, SCREEN_WIDTH},
};

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let mut buffer = DisplayBuffer::new();

    let text = embedded_graphics::text::Text::new(
        "Hello",
        Point::new(0, 7),
        MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
    );
    text.draw(&mut buffer).unwrap();

    let text = embedded_graphics::text::Text::new(
        "world!",
        Point::new(0, 7 + 8),
        MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
    );
    text.draw(&mut buffer).unwrap();
    let text_buffer_data = buffer.to_vec();
    let rows_to_update = (0..=SCREEN_HEIGHT as u8).into_iter().collect();

    display::write_region(
        (0, 0),
        (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
        text_buffer_data,
    )?;
    display::render(rows_to_update)?;
    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    // Nothing to do here
    Ok(())
}
