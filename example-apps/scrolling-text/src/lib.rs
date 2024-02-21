use embedded_graphics::{
    mono_font::{ascii::FONT_7X14, MonoFont, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};
use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, pack_monocolor_data, Color, MonocolorBuffer},
    kv_store,
};

const TEXT: &str = "HELLO WORLD FROM MEGABIT";
const FONT: &MonoFont = &FONT_7X14;

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let display_cfg = display::get_display_info()?;

    if display_cfg.is_rgb {
        display::set_monocolor_palette(Color::RED, Color::BLACK)?;
    }

    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    let display_cfg = display::get_display_info()?;
    let mut buffer = MonocolorBuffer::new(display_cfg.width, display_cfg.height);

    let bottom_y_point = (display_cfg.height - 1)
        - ((display_cfg.height - (FONT.character_size.height / 2) as usize) / 2);
    let bottom_y = i32::try_from(bottom_y_point).unwrap();
    let initial_offset = display_cfg.width as i32;

    let mut x_offset = kv_store::read::<i32>("x_offset")?.unwrap_or(-initial_offset);

    let text = embedded_graphics::text::Text::new(
        TEXT,
        Point::new(-x_offset, bottom_y),
        MonoTextStyle::new(FONT, BinaryColor::On),
    );
    text.draw(&mut buffer).unwrap();

    let text_width = FONT.character_size.width * TEXT.len() as u32
        + FONT.character_spacing * (TEXT.len() as u32 - 1);
    let max_offset = text_width + FONT.character_size.width;
    x_offset += 6;
    if x_offset >= max_offset as i32 {
        x_offset = -initial_offset;
    }

    kv_store::write("x_offset", x_offset)?;

    display::write_region(
        (0, 0),
        (display_cfg.width as u32, display_cfg.height as u32),
        pack_monocolor_data(buffer.get_data()),
    )?;
    display::render(0..display_cfg.height as u8)?;

    Ok(())
}
