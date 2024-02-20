use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};
use extism_pdk::*;
use megabit_app_sdk::display::{self, Color, MonocolorBuffer};

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let display_cfg = display::get_display_info()?;

    if display_cfg.is_rgb {
        display::set_monocolor_palette(Color::RED, Color::BLACK)?;
    }

    let mut buffer = MonocolorBuffer::new(display_cfg.width, display_cfg.height);

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
    let rows_to_update = (0..display_cfg.height as u8).into_iter().collect();
    let packed_data = buffer
        .get_data()
        .iter()
        .enumerate()
        .step_by(8)
        .map(|(idx, _)| {
            let mut byte = 0u8;
            for shift in 0..8 {
                if buffer.get_data()[idx + shift] {
                    byte |= 1 << (7 - shift);
                }
            }
            byte
        })
        .collect();

    display::write_region(
        (0, 0),
        (display_cfg.width as u32, display_cfg.height as u32),
        packed_data,
    )?;
    display::render(rows_to_update)?;
    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    // Nothing to do here
    Ok(())
}
